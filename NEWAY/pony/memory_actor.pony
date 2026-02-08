// NEWAY/pony/memory_actor.pony
// Distributed Actor-based Memory Coordinator

actor MemoryCoordinator
  let _id: String
  var _data: Map[String, String]

  new create(id: String) =>
    _id = id
    _data = Map[String, String]

  be store(key: String, value: String) =>
    _data(key) = value
    // Synchronize with other actors without locks

  be fetch(key: String, client: MemoryClient) =>
    try
      client.receive(_data(key))
    else
      client.not_found(key)

interface MemoryClient
  be receive(value: String)
  be not_found(key: String)
