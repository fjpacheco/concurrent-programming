use std::env;

use log::LevelFilter;

/// Constantes definidas mediante variables de entorno.
/// Expresado en gramos, pudiendo ser integer or float.
///
/// Un ejemplo de archivo `.env` para definir las variables de entorno podria ser:
///
/// ```txt
///     C_CACAO = "50.0"
///     A_AGUA_CALIENTE = "1000.0"
///     E_ESPUMA_LECHE = "10.0"
///     M_GRANOS_MOLIDOS = "10000.0"
///     L_LECHE_FRIA = "1000.0"
///     G_GRANOS = "1000.0"
///     N_DISPENSERS = "10.0"
/// ```
pub struct Consts;
impl Consts {
    /// Capacidad del contenedor de agua caliente obtenido de la variable de entorno A_AGUA_CALIENTE.
    /// Expresado en gramos, pudiendo ser integer or float.
    /// Por defecto 500.0
    pub fn a_agua_caliente() -> f32 {
        env::var("A_AGUA_CALIENTE")
            .unwrap_or("500.0".to_string())
            .parse::<f32>()
            .unwrap_or(500.0)
    }

    /// Capacidad del contenedor de cacao obtenido de la variable de entorno C_CACAO.
    /// Expresado en gramos, pudiendo ser integer or float.
    /// Por defecto 1000.0
    pub fn c_cacao() -> f32 {
        env::var("C_CACAO")
            .unwrap_or("1000.0".to_string())
            .parse::<f32>()
            .unwrap_or(1000.0)
    }

    /// Capacidad del contenedor de espuma de leche obtenido de la variable de entorno E_ESPUMA_LECHE.
    /// Expresado en gramos, pudiendo ser integer or float.
    /// Por defecto 700.0
    pub fn e_espuma_leche() -> f32 {
        env::var("E_ESPUMA_LECHE")
            .unwrap_or("700.0".to_string())
            .parse::<f32>()
            .unwrap_or(700.0)
    }

    /// Capacidad del contenedor de granos molidos obtenido de la variable de entorno M_GRANOS_MOLIDOS.
    /// Expresado en gramos, pudiendo ser integer or float.
    /// Por defecto 500.0
    pub fn m_granos_molidos() -> f32 {
        env::var("M_GRANOS_MOLIDOS")
            .unwrap_or("500.0".to_string())
            .parse::<f32>()
            .unwrap_or(500.0)
    }

    /// Cantidad de leche fria para recargar el contenedor E_ESPUMA_LECHE.
    /// Expresado en gramos, pudiendo ser integer or float.
    /// Por defecto 2000.0
    pub fn l_leche_fria() -> f32 {
        env::var("L_LECHE_FRIA")
            .unwrap_or("2000.0".to_string())
            .parse::<f32>()
            .unwrap_or(2000.0)
    }

    //// Cantidad de granos para recargar el contenedor M_GRANOS_MOLIDOS.
    /// Expresado en gramos, pudiendo ser integer or float.
    /// Por defecto 1000.0
    pub fn g_granos() -> f32 {
        env::var("G_GRANOS")
            .unwrap_or("1000.0".to_string())
            .parse::<f32>()
            .unwrap_or(1000.0)
    }

    /// Cantidad de threads dispensers a invocar.
    /// Como maximo se puede tener 1024 dispensers (`utils.rs: LIMIT_DISPENSERS`).
    /// Por defecto se invocan 8 dispensers.
    pub fn n_dispensers() -> usize {
        let n = env::var("N_DISPENSERS")
            .unwrap_or("8.0".to_string())
            .parse::<usize>()
            .unwrap_or(8);

        if n > LIMIT_DISPENSERS {
            LIMIT_DISPENSERS
        } else {
            n
        }
    }
}

/// Cantidad de segundos a esperar por cada gramo de ingrediente.
pub const SEGS_POR_GRAMO: f32 = 1.0;

/// Cantidad de segundos a esperar para recargar los contenedores
pub const SEGS_FOR_RELOAD: f32 = 10.0;

/// Cantidad maxima de dispensers a invocar.
pub const LIMIT_DISPENSERS: usize = 1024;

/// El sistema debe alertar por consola cuando los contenedores de granos,
/// leche y cacao se encuentran por debajo de X% de capacidad.
///
/// Este caso de uso se representa mediante la constante `X_ALERT_SYSTEM`.
pub const X_ALERT_SYSTEM: f32 = 0.10;

/// El tiempo en segundos que debe esperar el thread SYSTEM-ALERT para volver a alertar
/// sobre los estados de los contenedores.
pub const TIME_PERIODIC_ALERT: u64 = 10;

/// Inicializa el logger.
/// Lee la variable de entorno `RUST_LOG` para definir el nivel de log.
///
/// Por defecto el nivel de log es `INFO`.
pub fn init_logger() {
    env_logger::builder()
        .filter(
            None,
            env::var("RUST_LOG")
                .unwrap_or_default()
                .parse::<LevelFilter>()
                .unwrap_or(LevelFilter::Info),
        )
        .format_timestamp(None)
        .init();
}
