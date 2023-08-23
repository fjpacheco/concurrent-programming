use std::{io, net::UdpSocket};

use compartido::mensajes_cafetera::{Conectar, Desconectar, MensajeCafeteraBytes};

/// Proceso que recibe un tipo de mensaje -> conectar (c) o desconectar (d) y un número de nodo
/// y envía por udp el mensaje correspondiente
fn main() {
    let socket = UdpSocket::bind("127.0.0.1:1222").expect("Error al crear el socket");

    loop {
        println!("Ingrese la accion a realizar (d/c) seguido del ID_NODO a realizar la accion: ");
        let stdin = io::stdin();
        let mut _valores: Vec<&str> = vec![];
        let mut valor = String::new();

        stdin.read_line(&mut valor).expect("Error al leer el input");
        _valores = valor.split(' ').collect();
        if _valores.len() < 2 {
            println!("Necesito el tipo de mensaje (d/c) y el ID_NODO");
            continue;
        }
        let tipo: String = _valores[0].trim().to_string().to_lowercase();
        let nodo: String = _valores[1].trim().to_string().to_lowercase();

        let ip: String = "127.0.0.1:1235".to_string() + &nodo;

        if tipo == 'd'.to_string() {
            let msg = Desconectar::new(0, 0, 0).to_bytes();
            socket
                .send_to(&msg, ip)
                .expect("Error fatal al enviar el mensaje");
            println!("Envio DESCONECTAR al ID_NODO = {}", nodo);
        } else if tipo == 'c'.to_string() {
            let msg = Conectar::new(0, 0, 0).to_bytes();
            socket
                .send_to(&msg, ip)
                .expect("Error fatal al enviar el mensaje");
            println!("Envio CONECTAR al ID_NODO = {}", nodo);
        } else {
            println!("Mensaje desconocido");
            return;
        }
    }
}
