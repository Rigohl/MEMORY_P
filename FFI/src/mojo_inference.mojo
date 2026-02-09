// FFI/src/mojo_inference.mojo
// MOJO Inference Engine - Inferencia ultra-rápida

from tensor import Tensor
from algorithm import vectorize
from math import tanh, sqrt

struct InferenceEngine:
    var embedding_dim: Int
    var hidden_dim: Int
    
    fn __init__(inout self, embedding_dim: Int = 1536, hidden_dim: Int = 512):
        self.embedding_dim = embedding_dim
        self.hidden_dim = hidden_dim
    
    fn predict(self, input: Tensor[DType.float64]) -> Tensor[DType.float64]:
        """
        Realiza inferencia rápida sobre embedding de entrada
        """
        let output_dim = self.embedding_dim
        var output = Tensor[DType.float64](output_dim)
        
        # Capa 1: Transformación lineal + activación
        var hidden = self._linear_transform(input, self.hidden_dim)
        hidden = self._tanh_activation(hidden)
        
        # Capa 2: Proyección a espacio original
        output = self._linear_transform(hidden, output_dim)
        
        # Normalización
        output = self._normalize(output)
        
        return output
    
    fn _linear_transform(self, input: Tensor[DType.float64], output_dim: Int) -> Tensor[DType.float64]:
        """
        Transformación lineal simple (sin weights entrenados, usa transformación matemática)
        """
        var output = Tensor[DType.float64](output_dim)
        let input_dim = input.num_elements()
        
        # Guard against division by zero
        if input_dim == 0 or (input_dim + output_dim) == 0:
            return output
        
        for i in range(output_dim):
            var sum: Float64 = 0.0
            for j in range(input_dim):
                # Peso sintético basado en índices
                let weight = Float64((i + j + 1)) / Float64(input_dim + output_dim)
                sum += input[j] * weight
            
            # Safe division
            if input_dim > 0:
                output[i] = sum / Float64(input_dim)
            else:
                output[i] = 0.0
        
        return output
    
    fn _tanh_activation(self, input: Tensor[DType.float64]) -> Tensor[DType.float64]:
        """
        Activación tanh vectorizada
        """
        var output = Tensor[DType.float64](input.num_elements())
        
        @parameter
        fn tanh_op[width: Int](i: Int):
            output[i] = tanh(input[i])
        
        vectorize[tanh_op, 8](input.num_elements())
        
        return output
    
    fn _normalize(self, input: Tensor[DType.float64]) -> Tensor[DType.float64]:
        """
        Normalización L2
        """
        var output = Tensor[DType.float64](input.num_elements())
        var norm: Float64 = 0.0
        
        # Calcular norma
        for i in range(input.num_elements()):
            norm += input[i] * input[i]
        
        norm = sqrt(norm)
        
        if norm > 0.0:
            for i in range(input.num_elements()):
                output[i] = input[i] / norm
        else:
            output = input
        
        return output
    
    fn batch_predict(self, inputs: Tensor[DType.float64], batch_size: Int) -> Tensor[DType.float64]:
        """
        Predicción en batch para máxima performance
        """
        let output_size = batch_size * self.embedding_dim
        var outputs = Tensor[DType.float64](output_size)
        
        for i in range(batch_size):
            let start_idx = i * self.embedding_dim
            let end_idx = start_idx + self.embedding_dim
            
            # Extraer input individual
            var single_input = Tensor[DType.float64](self.embedding_dim)
            for j in range(self.embedding_dim):
                single_input[j] = inputs[start_idx + j]
            
            # Predecir
            let prediction = self.predict(single_input)
            
            # Copiar a output
            for j in range(self.embedding_dim):
                outputs[start_idx + j] = prediction[j]
        
        return outputs


# FFI-compatible functions

@export
fn mojo_inference_create(embedding_dim: Int, hidden_dim: Int) -> UnsafePointer[InferenceEngine]:
    """
    Crea instancia de InferenceEngine para FFI
    """
    let engine = InferenceEngine(embedding_dim, hidden_dim)
    let ptr = UnsafePointer[InferenceEngine].alloc(1)
    ptr.store(engine)
    return ptr

@export
fn mojo_inference_predict(
    engine_ptr: UnsafePointer[InferenceEngine],
    input_ptr: UnsafePointer[Float64],
    input_len: Int,
    output_ptr: UnsafePointer[Float64]
) -> Int:
    """
    Ejecuta predicción desde FFI
    Returns: 0 = success, 1 = error
    """
    try:
        let engine = engine_ptr.load()
        
        # Copiar input a Tensor
        var input = Tensor[DType.float64](input_len)
        for i in range(input_len):
            input[i] = input_ptr[i]
        
        # Predecir
        let output = engine.predict(input)
        
        # Copiar output
        for i in range(output.num_elements()):
            output_ptr[i] = output[i]
        
        return 0
    except:
        return 1

@export
fn mojo_inference_destroy(engine_ptr: UnsafePointer[InferenceEngine]):
    """
    Libera memoria de InferenceEngine
    """
    engine_ptr.free()
