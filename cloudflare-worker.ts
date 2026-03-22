/**
 * MEMORY_P MCP Gateway Worker
 * Routes JSON-RPC 2.0 requests to 19 microservice binaries
 * Deployed on Cloudflare Workers
 */

interface Env {
    BINARIES: KVNamespace;
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
 * Route incoming MCP JSON-RPC requests to appropriate microservice
 */
export default {
    async fetch(request: Request, env: Env): Promise<Response> {
        // CORS headers
        const corsHeaders = {
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
            "Access-Control-Allow-Headers": "Content-Type",
        };

        if (request.method === "OPTIONS") {
            return new Response(null, { headers: corsHeaders });
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
                const status = await response.json();

                // Store in KV for monitoring
                await env.BINARIES.put(
                    `health:${motorName}`,
                    JSON.stringify({
                        motor: motorName,
                        status: status.result?.status || "unknown",
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
