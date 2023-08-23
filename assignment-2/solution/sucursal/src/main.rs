extern crate serde;
extern crate serde_json;
use compartido::mensajes_cafetera::{
    Error, MensajeCafetera, MensajeCafeteraBytes, OkeyToCafetera, Ping, Restar, Sumar,
};
use rand::Rng;
use serde::Deserialize;
use sucursal::utils::{CANTIDAD_CAFETERAS, TIMEOUT, TIEMPO_DE_PREPARACION, PROBABILIDAD_ERROR};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::UdpSocket;
use std::time::Duration;
use std::{env, thread};
use sucursal::error_sucursal::{ErrorSucursal, TipoError};


#[derive(Deserialize)]
/// Estructura que define un unico pedido para una cafetera
struct Pedido {
    /// id correspondiente a la tarjeta del usuario
    id_cuenta: u32,
    /// tipo de pedido a realizar (suma o resta de puntos)
    tipo: String,
    /// cantidad de puntos a ser sumados o restados
    cantidad: u32,
}

/// Funcion que se invoca una vez finalizada la preparación de un cafe espera por la confirmación de la
/// sucursal para ser entregado, en caso de recibir error lo desecha
fn esperar_confirmacion(
    socket: UdpSocket,
    id_cuenta: u32,
    _cantidad: u32,
    id_cafetera: u8,
    id_nodo: String,
    multiplicador_timeout: u64,
) -> Result<bool, ErrorSucursal> {
    socket
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT * multiplicador_timeout)))
        .map_err(|x| ErrorSucursal::new(&x.to_string(), TipoError::ErrorGenerico))?;

    let mut buffer = [0u8; 14];
    match socket.recv_from(&mut buffer) {
        Ok((_, _addr)) => {
            let aux = buffer.to_vec();
            let tipo_mensaje = MensajeCafetera::from_bytes(aux[0]);

            if let MensajeCafetera::OKEY = tipo_mensaje {
                println!("El cafe fue entregado correctamente");
            } else {
                println!("El cafe fue desechado correctamente");
            }
        }
        Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
            // Envio ping, por que se pasó el timeout
            println!(
                "Soy cafetera {}, hubo un timeout al esperar confirmacion, envió ping",
                id_cafetera
            );

            let ping = Ping::new(id_cafetera, id_cuenta, 0).to_bytes();
            socket
                .send_to(&ping, "127.0.0.1:1235".to_owned() + &id_nodo)
                .map_err(|x| ErrorSucursal::new(&x.to_string(), TipoError::ErrorGenerico))?;

            println!("Ya envié el ping, vuelvo a esperar por un Ok o Err de confirmacion");
            return Ok(true);
        }
        Err(err) => {
            // Error al leer del socket, paso al siguiente pedido
            println!("Error reading from socket: {}", err);
        }
    }
    Ok(false)
}

/// Funcion que se invoca por una cafetera luego de que se envie el pedido actual se esperan las correspondientes
/// respuestas o se puede producir un timeout que invoca un ping. La preparación del cafe puede fallar con una
/// probabilidad dada por PROBABILIDAD_ERROR y el tiempo de preparacion del mismo es de TIEMPO_DE_PREPARACION
fn escuchar_respuesta(
    socket: UdpSocket,
    id_cuenta: u32,
    cantidad: u32,
    id_cafetera: u8,
    id_nodo: String,
    multiplicador_timeout: u64,
    tipo: String,
) -> Result<bool, ErrorSucursal> {
    socket
        .set_read_timeout(Some(Duration::from_secs(TIMEOUT * multiplicador_timeout)))
        .map_err(|x| ErrorSucursal::new(&x.to_string(), TipoError::ErrorGenerico))?;

    let mut buffer = [0u8; 14];

    match socket.recv_from(&mut buffer) {
        Ok((_, _addr)) => {
            // Se recibió una respuesta, debería ser un OK
            let aux = buffer.to_vec();
            let tipo_mensaje = MensajeCafetera::from_bytes(aux[0]);
            println!(
                "Recibí un {:?} de la cafetera.",
                tipo_mensaje
            );
            if let MensajeCafetera::OKEY = tipo_mensaje {
                //Recibo un ok, tengo que preparar el cafe y puede fallar...
                let numero_random: f64 = rand::thread_rng().gen();

                //Simulo que preparo el cafe
                thread::sleep(Duration::from_secs(TIEMPO_DE_PREPARACION));

                let mut _paquete: Vec<u8> = vec![];
                //verificar si hubo un error
                if numero_random < PROBABILIDAD_ERROR {
                    println!("Error producido en la cafetera {}", id_cafetera);
                    let mensaje_error = Error::new(id_cafetera, id_cuenta, 0);
                    _paquete = mensaje_error.to_bytes();
                    socket
                        .send_to(&_paquete, "127.0.0.1:1235".to_owned() + &id_nodo)
                        .map_err(|x| {
                            ErrorSucursal::new(&x.to_string(), TipoError::ErrorGenerico)
                        })?;
                } else {
                    println!(
                        "El café se termino de preparar en la cafetera {}",
                        id_cafetera
                    );
                    let mensaje_ok = OkeyToCafetera::new(id_cafetera, id_cuenta, cantidad);
                    _paquete = mensaje_ok.to_bytes();
                    socket
                        .send_to(&_paquete, "127.0.0.1:1235".to_owned() + &id_nodo)
                        .map_err(|x| {
                            ErrorSucursal::new(&x.to_string(), TipoError::ErrorGenerico)
                        })?;
                    if tipo == "RESTA" {
                        // Esperamos un Ok, si recibimos Err el cafe no es entregado
                        let mut repetir_confirmacion: bool = true;
                        let mut multiplicador_timeout_confirmacion: u64 = 1;

                        while repetir_confirmacion {
                            repetir_confirmacion = esperar_confirmacion(
                                socket.try_clone().expect("Error al clonar el socket"),
                                id_cuenta,
                                cantidad,
                                id_cafetera,
                                id_nodo.clone(),
                                multiplicador_timeout_confirmacion,
                            )?;
                            multiplicador_timeout_confirmacion += 1;
                        }
                    }
                }
            } else {
                println!("No se pudo ejecutar el pedido");
            }
        }
        Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
            // Envio ping, por que se pasó el timeout
            println!("Soy cafetera {}, hubo un timeout, envió ping", id_cafetera);

            let ping = Ping::new(id_cafetera, id_cuenta, 0).to_bytes();
            socket
                .send_to(&ping, "127.0.0.1:1235".to_owned() + &id_nodo)
                .map_err(|x| ErrorSucursal::new(&x.to_string(), TipoError::ErrorGenerico))?;

            println!("Ya envié el ping, vuelvo a esperar por un Ok o Err");
            return Ok(true);
        }
        Err(err) => {
            // Error al leer del socket, paso al siguiente pedido
            println!("Error reading from socket: {}", err);
        }
    }
    Ok(false)
}

/// Funcion que se encarga de procesar los pedidos de cada cafetera
/// invoca un socket por udp con direccion unica, que se genera con el id de nodo y cafetera
fn process_sublist(
    pedidos: Vec<String>,
    id_nodo: String,
    id_cafetera: u8,
) -> Result<(), ErrorSucursal> {
    let socket = UdpSocket::bind("127.0.0.1:57".to_owned() + &id_nodo + &id_cafetera.to_string())
    .map_err(|x| ErrorSucursal::new(&x.to_string(), TipoError::ErrorConexion))?;
    
    println!("Soy la cafetera {} y voy a procesar {} pedidos", id_cafetera, pedidos.len());
    
    // Cada pedido de la cafetera es procesado
    for p in pedidos {
        let pedido: Pedido = serde_json::from_str(&p)
            .map_err(|x| ErrorSucursal::new(&x.to_string(), TipoError::ErrorGenerico))?;
        let mut _msg: Vec<u8> = vec![];
        // a partir del tipo envio el mensaje correspondiente por udp
        if pedido.tipo == "SUMA" {
            _msg = Sumar::new(id_cafetera, pedido.id_cuenta, pedido.cantidad).to_bytes();
        } else if pedido.tipo == "RESTA" {
            _msg = Restar::new(id_cafetera, pedido.id_cuenta, pedido.cantidad).to_bytes();
        } else {
            continue;
        }
        println!(
            "Soy Cafetera {}, envió {:?} al nodo id {:?}",
            id_cafetera, pedido.tipo, id_nodo
        );

        let mut repetir_pedido: bool = true;
        let mut multiplicador_timeout: u64 = 1;
        socket
            .send_to(&_msg, "127.0.0.1:1235".to_owned() + &id_nodo)
            .map_err(|x| ErrorSucursal::new(&x.to_string(), TipoError::ErrorGenerico))?;

        while repetir_pedido {
            repetir_pedido = escuchar_respuesta(
                socket.try_clone().expect("Error al clonar el socket"),
                pedido.id_cuenta,
                pedido.cantidad,
                id_cafetera,
                id_nodo.clone(),
                multiplicador_timeout,
                pedido.tipo.clone(),
            )?;
            multiplicador_timeout += 1;
        }
    }
    Ok(())
}

/// Procesamiento de la lista de pedidos por parte de las cafeteras, recibimos por parámetros nuestro numero de ID_NODO
/// al que se conectara la sucursal, y tambien el nombre del archivo de pedidos que procesara la sucursal.
/// Los pedidos serán divididos entre las cafeteras, según la cantidad dada por la constante CANTIDAD_CAFETERAS y
/// cada procesamiento de cafetera se ejecutara en su propio thread
fn main() -> Result<(), ErrorSucursal> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        return Err(ErrorSucursal::new(
            "Es necesario recibir el numero de ID_NODO al que se conectara la sucursal, y tambien el nombre del archivo de pedidos. Ejemplo: procesando los pedidos (indicados en pedidos.txt) en el servidor con ID_NODO = 1: ´cargo run -- 1 pedidos.txt´",
            TipoError::ErrorArgs,
        ));
    }
    let id_nodo: String = args[1].to_string();
    let pedidos_file: String = args[2].to_string();

    // let path_pedidos = "Pedidos/pedidos_sucursal".to_string() + &id_nodo + ".txt";
    let path_pedidos = "Pedidos/".to_string() + &pedidos_file;
    let file = File::open(path_pedidos)
        .map_err(|x| ErrorSucursal::new(&x.to_string(), TipoError::ErrorArchivo))?;

    let reader = BufReader::new(file);

    let mut lines = Vec::new();

    //Pasar todos los pedidos a una lista
    for line in reader.lines().flatten() {
        lines.push(line);
    }

    let mut cafeteras = CANTIDAD_CAFETERAS;
    if CANTIDAD_CAFETERAS > lines.len() {
        cafeteras = lines.len();
    }

    let cantidad_por_cafetera = lines.len() / cafeteras;

    let mut thread_handles = vec![];

    for i in 0..cafeteras {
        let inicio = i * cantidad_por_cafetera;
        let rango_final = if i == cafeteras - 1 {
            lines.len()
        } else {
            (i + 1) * cantidad_por_cafetera
        };

        let sublista = lines[inicio..rango_final].to_vec();

        let id_nodo_cpy = id_nodo.clone();
        thread_handles.push(thread::spawn(move || {
            process_sublist(sublista, id_nodo_cpy, i as u8)
        }));
    }

    for handle in thread_handles {
        let _ = handle
            .join()
            .map_err(|_x| ErrorSucursal::new("Error en join threads", TipoError::ErrorArchivo))?;
    }
    Ok(())
}
