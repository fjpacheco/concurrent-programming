use server::error_server::ErrorServer;
use server::nodo::Nodo;
use server::utils::ID_CORDINADOR_INICIAL;
use server::{coordinador::Coordinador, utils::CANT_MAX_NODOS};
use std::{env, thread};
use tokio::net::TcpListener;

async fn empezar_cordinador(listener: TcpListener) {
    let _res = Coordinador::start_listener(listener).await;
}

async fn empezar_nodo(id: u8) {
    let _res = Nodo::start(id, ID_CORDINADOR_INICIAL).await;
}

#[actix_rt::main]
async fn main() -> Result<(), ErrorServer> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(ErrorServer::new(
            "Falta el id del nodo",
            server::error_server::TipoError::ErrorArgs,
        ));
    }

    let id: u8 = args[1].parse().map_err(|_| {
        ErrorServer::new(
            "El id del nodo debe ser un numero",
            server::error_server::TipoError::ErrorArgs,
        )
    })?;

    if !(1..=CANT_MAX_NODOS).contains(&id) {
        println!("[SYSTEM] Rechazo conexion de ID_NODO = {:?}", id);
        println!("[SYSTEM] ID_NODO permitidos de 1 a {:?}", CANT_MAX_NODOS);
        return Ok(());
    }

    let mut _coordinador = None;
    let tcp_listener = Coordinador::create_listener(id).await?;

    _coordinador = Some(thread::spawn(move || empezar_cordinador(tcp_listener)));

    empezar_nodo(id).await;

    if let Some(coordinador) = _coordinador {
        coordinador
            .join()
            .map_err(|_| {
                ErrorServer::new(
                    "Error al esperar al coordinador",
                    server::error_server::TipoError::ErrorJoinThreads,
                )
            })?
            .await;
    }

    Ok(())
}
