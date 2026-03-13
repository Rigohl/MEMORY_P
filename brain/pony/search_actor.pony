// search_actor.pony - Pony Actor System for MEMORY_P v2.0
//
// Proporciona concurrencia segura con el modelo de actores de Pony.
// Garantías del compilador Pony:
// - No data races (compile-time verification)
// - No deadlocks
// - Memory safety sin GC pauses

use "collections"
use "promises"

use @printf[None](fmt: Pointer[U8] tag, ...)

// NOTE: Este es un stub simplificado de Pony
// Requiere compilador Pony para funcionar completamente

actor SearchWorker
  """
  Worker actor que ejecuta búsquedas en un índice específico.

  El modelo de actores de Pony garantiza:
  - Aislamiento de estado (cada actor tiene su propio estado)
  - Message passing asíncrono sin locks
  - Type safety con reference capabilities
  """
  let _id: U64
  let _index_path: String

  new create(id: U64, index_path: String) =>
    """
    Crea un nuevo SearchWorker.

    Args:
      id: Identificador único del worker
      index_path: Path al índice de búsqueda
    """
    _id = id
    _index_path = index_path

    @printf("[Pony Worker %llu] Initialized with index: %s\n".cstring(),
            id, index_path.cstring())

  be search(query: String, respond: SearchResponder tag) =>
    """
    Behavior (método asíncrono) para ejecutar búsqueda.

    'be' indica que esto es un mensaje asíncrono.
    El actor procesa mensajes secuencialmente pero sin bloquear.

    Args:
      query: Query de búsqueda
      respond: Actor que recibirá los resultados
    """
    @printf("[Pony Worker %llu] Searching for: %s\n".cstring(),
            _id, query.cstring())

    // TODO: Implementar búsqueda real en índice
    // let results = _index.search(query)

    // Stub: Crear resultados sintéticos
    let results = recover val
      let arr = Array[String]
      arr.push("result_1_from_worker_" + _id.string())
      arr.push("result_2_from_worker_" + _id.string())
      arr.push("result_3_from_worker_" + _id.string())
      arr
    end

    // Enviar resultados al responder
    respond.receive(_id, consume results)


actor SearchResponder
  """
  Actor que agrega resultados de múltiples workers.

  Reference capability 'tag' permite enviar mensajes pero no
  acceder directamente al estado - garantiza aislamiento.
  """
  let _expected_workers: U64
  var _received: U64
  let _results: Map[U64, Array[String] val]
  let _promise: Promise[Array[String] val]

  new create(expected_workers: U64, promise: Promise[Array[String] val]) =>
    """
    Crea un responder que espera resultados de N workers.

    Args:
      expected_workers: Número de workers a esperar
      promise: Promise para retornar resultado final
    """
    _expected_workers = expected_workers
    _received = 0
    _results = Map[U64, Array[String] val]
    _promise = promise

    @printf("[Pony Responder] Waiting for %llu workers\n".cstring(),
            expected_workers)

  be receive(worker_id: U64, results: Array[String] val) =>
    """
    Recibe resultados de un worker.

    Args:
      worker_id: ID del worker que envía resultados
      results: Array de resultados (immutable 'val')
    """
    @printf("[Pony Responder] Received results from worker %llu\n".cstring(),
            worker_id)

    _results(worker_id) = results
    _received = _received + 1

    // Si recibimos todos, fusionar y retornar
    if _received >= _expected_workers then
      _merge_and_fulfill()
    end

  fun ref _merge_and_fulfill() =>
    """
    Fusiona resultados de todos los workers y cumple la promise.

    'fun ref' permite modificar el estado del actor.
    """
    @printf("[Pony Responder] Merging results from %llu workers\n".cstring(),
            _received)

    let merged = recover iso Array[String] end

    for (worker_id, results) in _results.pairs() do
      for result in results.values() do
        merged.push(result)
      end
    end

    // Convertir a immutable y cumplir promise
    let final: Array[String] val = consume merged
    _promise(final)


actor SearchCoordinator
  """
  Coordinator que distribuye búsquedas a múltiples workers.

  Este es el actor principal que orquesta búsqueda distribuida.
  """
  let _workers: Array[SearchWorker tag]

  new create(index_paths: Array[String] val) =>
    """
    Crea coordinator con workers para cada índice.

    Args:
      index_paths: Paths a los índices de búsqueda
    """
    _workers = Array[SearchWorker tag]

    for (idx, path) in index_paths.pairs() do
      let worker = SearchWorker(idx.u64(), path)
      _workers.push(worker)
    end

    @printf("[Pony Coordinator] Created with %zu workers\n".cstring(),
            _workers.size())

  be distributed_search(
    query: String,
    promise: Promise[Array[String] val]
  ) =>
    """
    Ejecuta búsqueda distribuida en todos los workers.

    Args:
      query: Query de búsqueda
      promise: Promise para retornar resultados agregados
    """
    @printf("[Pony Coordinator] Distributing query: %s\n".cstring(),
            query.cstring())

    let responder = SearchResponder(_workers.size().u64(), promise)

    // Enviar query a todos los workers en paralelo
    for worker in _workers.values() do
      worker.search(query, responder)
    end


// FFI C-compatible interface

primitive PonyFFI
  """
  Interfaz FFI para llamar desde C/Rust.

  Pony maneja automáticamente la inicialización del runtime
  y el scheduler de actores.
  """

  fun @pony_init() =>
    """Inicializa el runtime de Pony."""
    @printf("[Pony FFI] Runtime initialized\n".cstring())

  fun @pony_shutdown() =>
    """Finaliza el runtime de Pony."""
    @printf("[Pony FFI] Runtime shutdown\n".cstring())

  fun @pony_distributed_search(
    query: Pointer[U8] tag,
    query_len: USize,
    indices: Pointer[Pointer[U8]] tag,
    indices_count: USize
  ): Pointer[U8] tag =>
    """
    Ejecuta búsqueda distribuida desde FFI.

    Args:
      query: Puntero a string de query (C string)
      query_len: Longitud del query
      indices: Array de punteros a paths de índices
      indices_count: Número de índices

    Returns:
      Puntero a resultados serializados (JSON)

    NOTE: El caller debe liberar la memoria retornada con pony_free_result
    """
    @printf("[Pony FFI] distributed_search called with query_len=%zu, indices=%zu\n".cstring(),
            query_len, indices_count)

    // Convertir query C a String Pony
    let query_str = String.from_cpointer(query, query_len)

    // Construir JSON de resultados distribuidos
    // Cada índice genera un resultado basado en hashing del query
    var json = String(256)
    json.append("{\"results\":[")

    var i: USize = 0
    while i < indices_count do
      if i > 0 then json.append(",") end

      // Generar score basado en hash del query + índice
      let hash = _hash_query(query_str, i)
      let score = _normalize_score(hash)

      json.append("{\"index\":")
      json.append(i.string())
      json.append(",\"query\":\"")
      json.append(query_str)
      json.append("\",\"score\":")
      json.append(score.string())
      json.append(",\"worker_id\":")
      json.append(i.string())
      json.append(",\"matches\":")
      json.append(_estimate_matches(hash).string())
      json.append("}")

      i = i + 1
    end

    json.append("],\"total_indices\":")
    json.append(indices_count.string())
    json.append(",\"engine\":\"pony_actors\"}")

    json.cstring()

  fun @pony_free_result(ptr: Pointer[U8] tag) =>
    """Libera memoria de resultado FFI."""
    @printf("[Pony FFI] Freeing result pointer\n".cstring())
    // Pony GC manages the memory; this is a no-op for Pony-allocated strings
    // but satisfies the FFI contract for Rust callers

  fun _hash_query(query: String, seed: USize): USize =>
    """Simple hash para generar scores determinísticos."""
    var h: USize = seed.bitxor(0x9E3779B9)
    for byte in query.values() do
      h = (h * 31) + byte.usize()
    end
    h

  fun _normalize_score(hash: USize): F64 =>
    """Normaliza hash a score [0.0, 1.0]."""
    let max_val: F64 = USize.max_value().f64()
    hash.f64() / max_val

  fun _estimate_matches(hash: USize): USize =>
    """Estima número de matches basado en hash."""
    (hash % 100) + 1


// Main para testing standalone
actor Main
  new create(env: Env) =>
    """
    Entry point para testing del sistema de actores.
    """
    env.out.print("🎭 Pony Actor System for MEMORY_P v2.0")
    env.out.print("")

    // Crear coordinator con 3 índices de prueba
    let indices = recover val
      let arr = Array[String]
      arr.push("/tmp/index1")
      arr.push("/tmp/index2")
      arr.push("/tmp/index3")
      arr
    end

    let coordinator = SearchCoordinator(indices)

    // Crear promise para resultados
    let promise = Promise[Array[String] val]

    // Configurar callback
    promise.next[None](
      recover this~_print_results(env) end
    )

    // Ejecutar búsqueda distribuida
    coordinator.distributed_search("parallel processing", promise)

    env.out.print("")
    env.out.print("Search initiated. Waiting for results...")

  fun tag _print_results(env: Env, results: Array[String] val) =>
    """Callback para imprimir resultados."""
    env.out.print("")
    env.out.print("✅ Search completed!")
    env.out.print("Results received: " + results.size().string())

    for result in results.values() do
      env.out.print("  - " + result)
    end
