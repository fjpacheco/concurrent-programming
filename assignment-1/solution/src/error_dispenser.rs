use crate::enums::ErrorType;

/// Estructura para manejar los errores de la cafeteria
#[derive(Debug)]
pub struct ErrorCafeteria {
    /// Mensaje de error
    pub mensaje: String,

    /// Tipo de error
    pub type_error: ErrorType,
}

impl ErrorCafeteria {
    /// Crea la cafeteria con un mensaje de error y un tipo de error generico.
    pub fn new(mensaje: &str) -> Self {
        ErrorCafeteria {
            mensaje: mensaje.to_string(),
            type_error: ErrorType::ErrorGeneric,
        }
    }

    /// Crea la cafeteria con un mensaje de error y un tipo de error especifico.
    pub fn new_of_type(mensaje: &str, type_error: ErrorType) -> Self {
        ErrorCafeteria {
            mensaje: mensaje.to_string(),
            type_error,
        }
    }
}
