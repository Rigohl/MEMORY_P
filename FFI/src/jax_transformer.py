import jax
import jax.numpy as jnp
from jax import jit, vmap
import numpy as np

def init_transformer_params(rng, embed_dim, num_heads):
    k1, k2, k3 = jax.random.split(rng, 3)
    return {
        "wq": jax.random.normal(k1, (embed_dim, embed_dim)),
        "wk": jax.random.normal(k2, (embed_dim, embed_dim)),
        "wv": jax.random.normal(k3, (embed_dim, embed_dim)),
        "wo": jax.random.normal(rng, (embed_dim, embed_dim)),
    }

@jit
def mha(params, x):
    # Simplified Multi-Head Attention
    q = jnp.dot(x, params["wq"])
    k = jnp.dot(x, params["wk"])
    v = jnp.dot(x, params["wv"])

    # Scale dot-product attention
    d_k = q.shape[-1]
    scores = jnp.dot(q, k.T) / jnp.sqrt(d_k)
    weights = jax.nn.softmax(scores)
    attn = jnp.dot(weights, v)

    return jnp.dot(attn, params["wo"])

@jit
def predict_next_step(params, current_state):
    # Predict next embedding state
    attn_out = mha(params, current_state)
    next_state = current_state + attn_out # Residual
    return next_state / (jnp.linalg.norm(next_state) + 1e-8)
