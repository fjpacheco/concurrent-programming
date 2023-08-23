/// Tipos de errores que pueden ocurrir en el servidor
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TipoError {
    ErrorGenerico,
    ErrorConexion,
    ErrorJoinThreads,
    ErrorArgs,
}
/// Estructura para manejar los errores del servidor
#[derive(Debug)]
pub struct ErrorServer {
    /// Mensaje de error
    pub mensaje: String,

    /// Tipo de error
    pub tipo_error: TipoError,
}

impl ErrorServer {
    /// Crea un nuevo mensaje de Error
    pub fn new(mensaje: &str, tipo_error: TipoError) -> Self {
        ErrorServer {
            mensaje: mensaje.to_string(),
            tipo_error,
        }
    }
}
