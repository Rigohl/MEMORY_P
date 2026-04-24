/**
 * MEMORY_P MCP Gateway Worker
 * Routes JSON-RPC 2.0 requests to 19 microservice binaries
 * Deployed on Cloudflare Workers
 */

interface Env {
    BINARIES: KVNamespace;
    MEMORY_P_API_KEY?: string;
    JWT_SECRET?: string;
    OAUTH_CLIENT_ID?: string;
    OAUTH_CLIENT_SECRET?: string;
}

interface JsonRpcRequest {
    jsonrpc: "2.0";
    id: number;
    method: string;
    params?: Record<string, unknown>;
}

interface JsonRpcResponse<T> {
    jsonrpc: "2.0";
    id: number;
    result?: T;
    error?: { code: number; message: string; data?: unknown };
}

interface OAuthCode {
    code: string;
    clientId: string;
    redirectUri: string;
    codeChallenge: string;
    expiresAt: number;
    scope: string;
}

interface JWTPayload {
    sub: string;
    iat: number;
    exp: number;
    scope: string;
}

// 19 microservices mapping
const MOTORS_MAP: Record<string, { port: number; name: string }> = {
    // Core engines
    qdrant: { port: 3010, name: "qdrant_search_engine" },
    faiss: { port: 3011, name: "faiss_search_engine" },
    scann: { port: 3012, name: "scann_search_engine" },
    tantivy: { port: 3013, name: "tantivy_engine" },
    lnx: { port: 3014, name: "lnx_cluster_engine" },
    meilisearch: { port: 3015, name: "meilisearch_search_engine" },
    memorybank: { port: 3016, name: "memorybank_orchestrator" },

    // FFI specialized
    mojo: { port: 3017, name: "mojo_search_engine" },
    pony: { port: 3018, name: "pony_actor_engine" },
    jax: { port: 3019, name: "jax_ml_engine" },

    // Additional utilities
    julia: { port: 3020, name: "julia_optimization_engine" },
    chaos: { port: 3021, name: "chaos_analyzer" },

    // Original suite
    memory_p: { port: 3022, name: "memory_p" },
    mcp_server: { port: 3023, name: "mcp_server" },
    motor_orchestrator: { port: 3024, name: "motor_orchestrator" },
    jar: { port: 3025, name: "jar" },

    // Tier engines
    vector: { port: 3026, name: "vector_engine" },
    text: { port: 3027, name: "text_engine" },
    specialized: { port: 3028, name: "specialized_engine" },
};

/**
 * Encode string to base64url (for PKCE)
 */
function toBase64Url(str: string): string {
    return btoa(str).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

/**
 * Decode base64url string to original string
 */
function fromBase64Url(str: string): string {
    // Add padding if missing
    let base64 = str.replace(/-/g, "+").replace(/_/g, "/");
    while (base64.length % 4 !== 0) {
        base64 += "=";
    }
    return atob(base64);
}

/**
 * Generate random string (for code_verifier)
 */
function generateRandomString(length: number = 43): string {
    const charset = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let result = "";
    const array = new Uint8Array(length);
    if (typeof crypto !== "undefined" && crypto.getRandomValues) {
        crypto.getRandomValues(array);
    }
    for (let i = 0; i < length; i++) {
        result += charset.charCodeAt(array[i] % charset.length);
    }
    return result;
}

/**
 * Calculate PKCE code_challenge from code_verifier (SHA256)
 */
async function calculateCodeChallenge(verifier: string): Promise<string> {
    const encoder = new TextEncoder();
    const data = encoder.encode(verifier);
    const hashBuffer = await crypto.subtle.digest("SHA-256", data);
    return toBase64Url(String.fromCharCode(...new Uint8Array(hashBuffer)));
}

/**
 * Create JWT token
 */
async function createJWT(payload: JWTPayload, secret: string): Promise<string> {
    const header = { alg: "HS256", typ: "JWT" };
    const encoder = new TextEncoder();
    const toSign = `${btoa(JSON.stringify(header))}.${btoa(JSON.stringify(payload))}`;

    const keyData = encoder.encode(secret);
    const key = await crypto.subtle.importKey("raw", keyData, { name: "HMAC", hash: "SHA-256" }, false, ["sign"]);
    const signature = await crypto.subtle.sign("HMAC", key, encoder.encode(toSign));

    return `${toSign}.${toBase64Url(String.fromCharCode(...new Uint8Array(signature)))}`;
}

/**
 * Verify JWT token signature and expiration
 */
async function verifyJWT(token: string, secret: string): Promise<JWTPayload | null> {
    try {
        const [headerB64, payloadB64, signatureB64] = token.split(".");
        if (!headerB64 || !payloadB64 || !signatureB64) return null;

        // Verify signature using Web Crypto API
        const encoder = new TextEncoder();
        const data = encoder.encode(`${headerB64}.${payloadB64}`);
        const keyData = encoder.encode(secret);

        const key = await crypto.subtle.importKey(
            "raw",
            keyData,
            { name: "HMAC", hash: "SHA-256" },
            false,
            ["verify"]
        );

        // Convert base64url signature back to binary
        const signatureBin = Uint8Array.from(
            fromBase64Url(signatureB64),
            c => c.charCodeAt(0)
        );

        const isValid = await crypto.subtle.verify("HMAC", key, signatureBin, data);
        if (!isValid) return null;

        const payload = JSON.parse(fromBase64Url(payloadB64)) as JWTPayload;

        // Check expiration
        if (payload.exp < Math.floor(Date.now() / 1000)) {
            return null;
        }

        return payload;
    } catch {
        return null;
    }
}

/**
 * Authenticate request using API key or JWT
 */
async function authenticateRequest(request: Request, env: Env): Promise<{ valid: boolean; reason?: string }> {
    // Public endpoints
    const url = new URL(request.url);
    const pathname = url.pathname;
    if (pathname === "/health" || pathname.startsWith("/oauth/")) {
        return { valid: true };
    }

    // Get API key from environment (set in wrangler.toml)
    const expectedKey = env.MEMORY_P_API_KEY;

    if (!expectedKey) {
        return { valid: false, reason: "Server misconfiguration: MEMORY_P_API_KEY is not set" };
    }

    // Check Authorization header
    const authHeader = request.headers.get("Authorization");
    if (authHeader) {
        // Support "Bearer {token}" format
        const token = authHeader.replace("Bearer ", "").trim();
        if (token === expectedKey) {
            return { valid: true };
        }

        // Try JWT verification if configured
        if (env.JWT_SECRET) {
            const payload = await verifyJWT(token, env.JWT_SECRET);
            if (payload) {
                return { valid: true };
            }
        }
    }

    // Check X-API-Key header
    const apiKey = request.headers.get("X-API-Key");
    if (apiKey === expectedKey) {
        return { valid: true };
    }

    return { valid: false, reason: "Missing or invalid API key. Use -H 'X-API-Key: your-key' or -H 'Authorization: Bearer your-key'" };
}

/**
 * Route incoming MCP JSON-RPC requests to appropriate microservice
 */
export default {
    async fetch(request: Request, env: Env): Promise<Response> {
        // CORS headers
        const corsHeaders = {
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
            "Access-Control-Allow-Headers": "Content-Type, Authorization, X-API-Key",
        };

        if (request.method === "OPTIONS") {
            return new Response(null, { headers: corsHeaders });
        }

        // Authenticate request
        const auth = await authenticateRequest(request, env);
        if (!auth.valid) {
            return new Response(
                JSON.stringify({
                    jsonrpc: "2.0",
                    id: null,
                    error: {
                        code: -32000,
                        message: "Unauthorized",
                        data: auth.reason,
                    },
                }),
                { status: 401, headers: { "Content-Type": "application/json", ...corsHeaders } }
            );
        }

        const url = new URL(request.url);
        const method = request.method;

        // Health check endpoint
        if (url.pathname === "/health") {
            return new Response(
                JSON.stringify({
                    jsonrpc: "2.0",
                    id: 1,
                    result: {
                        status: "healthy",
                        motors_available: Object.keys(MOTORS_MAP).length,
                        timestamp: new Date().toISOString(),
                    },
                }),
                {
                    headers: { "Content-Type": "application/json", ...corsHeaders },
                }
            );
        }

        // OAuth 2.0 Endpoints
        // POST /oauth/authorize
        if (url.pathname === "/oauth/authorize" && method === "POST") {
            try {
                const data = await request.json() as any;
                const { client_id, redirect_uri, code_challenge, scope = "mcp:full" } = data;

                if (!client_id || !redirect_uri || !code_challenge) {
                    return new Response(
                        JSON.stringify({
                            error: "invalid_request",
                            error_description: "Missing required parameters: client_id, redirect_uri, code_challenge",
                        }),
                        { status: 400, headers: { "Content-Type": "application/json", ...corsHeaders } }
                    );
                }

                // Generate authorization code (valid for 60 seconds)
                const code = generateRandomString(32);
                const oauth: OAuthCode = {
                    code,
                    clientId: client_id,
                    redirectUri: redirect_uri,
                    codeChallenge: code_challenge,
                    scope,
                    expiresAt: Math.floor(Date.now() / 1000) + 60,
                };

                // Store code in KV
                await env.BINARIES.put(`oauth:code:${code}`, JSON.stringify(oauth), { expirationTtl: 60 });

                return new Response(
                    JSON.stringify({
                        authorization_code: code,
                        expires_in: 60,
                        redirect_uri: `${redirect_uri}?code=${code}&state=${url.searchParams.get("state") || ""}`,
                    }),
                    { status: 200, headers: { "Content-Type": "application/json", ...corsHeaders } }
                );
            } catch (error) {
                return new Response(
                    JSON.stringify({ error: "invalid_request", error_description: String(error) }),
                    { status: 400, headers: { "Content-Type": "application/json", ...corsHeaders } }
                );
            }
        }

        // POST /oauth/token
        if (url.pathname === "/oauth/token" && method === "POST") {
            try {
                const data = await request.json() as any;
                const { code, code_verifier, client_id, client_secret } = data;

                if (!code || !code_verifier || !client_id) {
                    return new Response(
                        JSON.stringify({
                            error: "invalid_request",
                            error_description: "Missing required parameters: code, code_verifier, client_id",
                        }),
                        { status: 400, headers: { "Content-Type": "application/json", ...corsHeaders } }
                    );
                }

                // Retrieve authorization code from KV
                const storedCode = await env.BINARIES.get(`oauth:code:${code}`);
                if (!storedCode) {
                    return new Response(
                        JSON.stringify({ error: "invalid_grant", error_description: "Authorization code not found or expired" }),
                        { status: 400, headers: { "Content-Type": "application/json", ...corsHeaders } }
                    );
                }

                const oauth = JSON.parse(storedCode) as OAuthCode;

                // Verify PKCE code_challenge
                const calculatedChallenge = await calculateCodeChallenge(code_verifier);
                if (calculatedChallenge !== oauth.codeChallenge) {
                    return new Response(
                        JSON.stringify({ error: "invalid_grant", error_description: "code_verifier does not match code_challenge" }),
                        { status: 400, headers: { "Content-Type": "application/json", ...corsHeaders } }
                    );
                }

                // Verify client credentials (optional in public clients)
                if (client_secret && client_secret !== env.OAUTH_CLIENT_SECRET) {
                    return new Response(
                        JSON.stringify({ error: "invalid_client", error_description: "Invalid client secret" }),
                        { status: 400, headers: { "Content-Type": "application/json", ...corsHeaders } }
                    );
                }

                // Create JWT token
                const secret = env.JWT_SECRET;
                if (!secret) {
                    return new Response(
                        JSON.stringify({ error: "server_error", error_description: "Server misconfiguration: JWT_SECRET is not set" }),
                        { status: 500, headers: { "Content-Type": "application/json", ...corsHeaders } }
                    );
                }

                const payload: JWTPayload = {
                    sub: client_id,
                    iat: Math.floor(Date.now() / 1000),
                    exp: Math.floor(Date.now() / 1000) + 3600, // 1 hour expiry
                    scope: oauth.scope,
                };

                const accessToken = await createJWT(payload, secret);

                // Cleanup - delete used code
                await env.BINARIES.delete(`oauth:code:${code}`);

                return new Response(
                    JSON.stringify({
                        access_token: accessToken,
                        token_type: "Bearer",
                        expires_in: 3600,
                        scope: oauth.scope,
                    }),
                    { status: 200, headers: { "Content-Type": "application/json", ...corsHeaders } }
                );
            } catch (error) {
                return new Response(
                    JSON.stringify({ error: "server_error", error_description: String(error) }),
                    { status: 500, headers: { "Content-Type": "application/json", ...corsHeaders } }
                );
            }
        }

        // Route /mcp/{motor}/{endpoint}
        const pathParts = url.pathname.split("/").filter(Boolean);

        if (pathParts[0] !== "mcp") {
            return new Response(
                JSON.stringify({
                    jsonrpc: "2.0",
                    id: null,
                    error: {
                        code: -32600,
                        message: "Invalid Request - use /mcp/{motor}/{endpoint}",
                    },
                }),
                { status: 400, headers: { "Content-Type": "application/json", ...corsHeaders } }
            );
        }

        const motorName = pathParts[1];
        const endpoint = pathParts[2] || "health";
        const motor = MOTORS_MAP[motorName];

        if (!motor) {
            return new Response(
                JSON.stringify({
                    jsonrpc: "2.0",
                    id: null,
                    error: {
                        code: -32601,
                        message: `Motor not found: ${motorName}. Available: ${Object.keys(MOTORS_MAP).join(", ")}`,
                    },
                }),
                { status: 404, headers: { "Content-Type": "application/json", ...corsHeaders } }
            );
        }

        // Forward request to local microservice
        try {
            const body = await request.json();
            const localUrl = `http://localhost:${motor.port}/mcp/${endpoint}`;

            const response = await fetch(localUrl, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(body),
            });

            const data = await response.json();

            return new Response(JSON.stringify(data), {
                status: response.status,
                headers: { "Content-Type": "application/json", ...corsHeaders },
            });
        } catch (error) {
            console.error(`Error routing to ${motor.name}:`, error);

            return new Response(
                JSON.stringify({
                    jsonrpc: "2.0",
                    id: null,
                    error: {
                        code: -32603,
                        message: "Internal error - motor unavailable",
                        data: String(error),
                    },
                }),
                {
                    status: 503,
                    headers: { "Content-Type": "application/json", ...corsHeaders },
                }
            );
        }
    },

    /**
     * Scheduled handler for health checks
     */
    async scheduled(event: ScheduledEvent, env: Env): Promise<void> {
        console.log("Checking motor health...");

        for (const [motorName, motor] of Object.entries(MOTORS_MAP)) {
            try {
                const response = await fetch(`http://localhost:${motor.port}/mcp/health`);
                const data = (await response.json()) as any;

                // Store in KV for monitoring
                await env.BINARIES.put(
                    `health:${motorName}`,
                    JSON.stringify({
                        motor: motorName,
                        status: data?.result?.status || "unknown",
                        timestamp: new Date().toISOString(),
                    })
                );

                console.log(`✓ ${motorName} healthy`);
            } catch (error) {
                console.error(`✗ ${motorName} error:`, error);

                await env.BINARIES.put(
                    `health:${motorName}`,
                    JSON.stringify({
                        motor: motorName,
                        status: "unhealthy",
                        error: String(error),
                        timestamp: new Date().toISOString(),
                    })
                );
            }
        }
    },
};
