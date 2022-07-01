use std::net::{IpAddr};
use std::path::Path;
use std::sync::{Arc};
use std::sync::atomic::Ordering;
use crate::{log_msg,LOG, LogLevel, MsgType, RUNTIME_NOTIFY, SocketAddrInfo, StringData, WebSocketCloseCallBack, WebSocketConnectedCallBack, WebSocketRecvCallBack, WebSocketSendCallBack};
use tokio_tungstenite::{tungstenite::{self,handshake::server::{Response as HandleShakeResponse, ErrorResponse},
                                      http::{Request as HandleShakeRequest, StatusCode}}, WebSocketStream};
use crate::{LogLevel::LLError,WebSocketMsg};
use futures_util::{StreamExt, SinkExt};
use std::fs::File;
use std::io::{self, BufReader};
use rustls_pemfile::{certs, rsa_private_keys};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_rustls::{TlsAcceptor,rustls::{self,Certificate, PrivateKey}};
use tokio_tungstenite::tungstenite::http::header::HeaderName;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

#[no_mangle]
pub extern "stdcall" fn get_header(header: StringData,req: usize,result: *mut StringData){
    let data = unsafe {
        let key = String::from_raw_parts(header.data as *mut u8,header.len,header.len);
        let req = req as *const HandleShakeRequest<()>;
        let req = &*req;
        let v = req.headers().get(key).unwrap();
        v.as_bytes()
    };
    unsafe {
        (*result).data = data.as_ptr() as usize;
        (*result).len = data.len();
    }
}

#[no_mangle]
pub extern "stdcall" fn request_path(req: usize,result: *mut StringData){
    let data = unsafe {
        let req = req as *const HandleShakeRequest<()>;
        let req = &*req;
        req.uri().path().as_bytes()
    };
    unsafe {
        (*result).data = data.as_ptr() as usize;
        (*result).len = data.len();
    }
}

#[no_mangle]
pub extern "stdcall" fn request_query(req: usize,result: *mut StringData){
    if let Some(data) = unsafe {
        let req = req as *const HandleShakeRequest<()>;
        let req = &*req;
        req.uri().query()
    }{
        unsafe {
            (*result).data = data.as_ptr() as usize;
            (*result).len = data.len();
        }
    }else{
        unsafe {
            (*result).data = 0 as usize;
            (*result).len = 0;
        }
    }

}

pub type VisitHeaderCallBack = extern "stdcall" fn(key: StringData,value: StringData,data: usize)->bool;

#[no_mangle]
pub extern "stdcall" fn visit_headers(req: usize,callback: VisitHeaderCallBack,data: usize){
    let req = unsafe{
        let req = req as *const HandleShakeRequest<()>;
        &*req
    };
    for (key,value) in req.headers().iter(){
        let key = key.as_str().as_bytes();
        let value = value.as_bytes();
        if !callback(StringData{
            data: key.as_ptr() as usize,
            len: key.len(),
        },StringData{
            data: value.as_ptr() as usize,
            len: value.len(),
        },data){
            return;
        }
    }
}

#[no_mangle]
pub extern "stdcall" fn set_resp_header(resp: usize,key: StringData,value: StringData){
    if key.data as usize == 0 || key.len == 0{
        return;
    }
    let resp = resp as *mut HandleShakeResponse;
    let (key,resp) = unsafe{
        (Vec::<u8>::from_raw_parts(key.data as *mut u8,key.len,key.len),&mut *resp)
    };
    let key = std::mem::ManuallyDrop::new(key);//外部的，不需要内部释放
    if let Ok(key) = HeaderName::from_bytes(key.as_slice()){
        if value.data  as usize == 0 || value.len == 0{
            resp.headers_mut().remove(key);
            return;
        }
        let value = unsafe{
            Vec::<u8>::from_raw_parts(value.data as *mut u8,value.len,value.len)
        };
        let value = std::mem::ManuallyDrop::new(value); //外部的，不需要内部释放
        if let Ok(value) = HeaderValue::from_bytes(value.as_slice()){
            resp.headers_mut().insert(key,value);
        }
    }
}

#[no_mangle]
pub extern "stdcall" fn set_resp_status(resp: usize,status_code: u16){
    let resp = unsafe {
        let resp = resp as *mut HandleShakeResponse;
        &mut *resp
    };
    {
        if resp.status().as_u16() == status_code{
            return;
        }
    }
    match StatusCode::from_u16(status_code){
        Ok(code) =>{
            *resp.status_mut() = code;
        },
        _=>(),
    }
}

pub type WebSocketHandleShake = extern "stdcall" fn(req: usize,response: usize,manager: usize)->bool;
pub type BeforeHandleShake = extern "stdcall" fn(peer: *const SocketAddrInfo,manager: usize)->bool;

#[repr(C)]
pub struct ServerCallBack{
    pub manager: usize,
    pub before_handle_shake: Option<BeforeHandleShake>,
    pub on_handle_shake: Option<WebSocketHandleShake>,
    pub on_connected: Option<WebSocketConnectedCallBack>,
    pub on_recv: Option<WebSocketRecvCallBack>,
    pub on_send: Option<WebSocketSendCallBack>,
    pub on_closed: Option<WebSocketCloseCallBack>,
}

#[repr(C)]
pub struct TlsFileInfo{
    pub cert_file: StringData,          //证书文件
    pub key_file: StringData,           //证书的秘钥文件
}

impl TlsFileInfo {
    pub fn default()->Self{
        Self{
            cert_file: StringData::empty(),
            key_file:  StringData::empty(),
        }
    }
}


async fn runtime_quit_notify()->(){
    unsafe{
        if let Some(ref tx) = RUNTIME_NOTIFY{
            let _ = tx.subscribe().changed().await;
            return;
        }
    }
    std::future::pending::<()>().await;
}


fn load_certs(path: &Path) -> std::io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    rsa_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}

#[no_mangle]
pub extern "stdcall" fn websocket_listen(port: u16, callback: ServerCallBack,tls: TlsFileInfo)->usize{
    if let None =  callback.on_connected{
        log_msg!(LogLevel::LLDebug, "未指定on_connected");
        return 0;
    }
    let mut ip_port = "0.0.0.0:".to_string();
    ip_port.push_str(&port.to_string());
    let (tx,rx) = tokio::sync::watch::channel(Option::<()>::None);
    let tx = Arc::new(tx);

    if tls.cert_file.data == 0 || tls.cert_file.len == 0 || tls.key_file.data == 0 || tls.key_file.len == 0{
        tokio::spawn(handle_listener(ip_port,callback,tx.clone(),rx,None));
        let tx = Box::new(tx);
        return Box::into_raw(tx) as *const Arc<tokio::sync::watch::Sender<Option<()>>> as usize;
    }
    let cert_file = unsafe{
        String::from_raw_parts(tls.cert_file.data as *mut u8,tls.cert_file.len,tls.cert_file.len)
    };
    let cert_file = std::mem::ManuallyDrop::new(cert_file);
    let key_file = unsafe{
        String::from_raw_parts(tls.key_file.data as *mut u8,tls.key_file.len,tls.key_file.len)
    };
    let key_file = std::mem::ManuallyDrop::new(key_file);
    let certs = {
        if let Ok(f) = load_certs(Path::new(&*cert_file)){
            f
        }else{
            return 0;
        }
    };
    let mut keys = {
        if let Ok(f) = load_keys(Path::new(&*key_file)){
            f
        }else{
            return 0;
        }
    };
    if let Ok(config) = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err)){
        let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(config));
        tokio::spawn(handle_listener(ip_port,callback,tx.clone(),rx,Some(acceptor)));
        let tx = Box::new(tx);
        return Box::into_raw(tx) as *const Arc<tokio::sync::watch::Sender<Option<()>>> as usize;
    }
    0
}

async fn handle_listener(ip_port: String,callback: ServerCallBack,tx: Arc<tokio::sync::watch::Sender<Option<()>>>,mut rx: tokio::sync::watch::Receiver<Option<()>>,acceptor: Option<TlsAcceptor>){
    log_msg!(LogLevel::LLDebug,"websocket start at {}",&ip_port);
    let listener = tokio::net::TcpListener::bind(&ip_port).await.unwrap();
    let callback = Arc::new(callback);
    loop {
        let accept = listener.accept();
        tokio::select! {
            _ = rx.changed()=>{
                //要停止了，关闭服务
                log_msg!(LogLevel::LLDebug,"websocket服务{}关闭",&ip_port);
                return;
            },
            _= runtime_quit_notify()=>{
                //整体退出了
                return;
            },
            tcp = accept=>{
                match tcp{
                    Err(e)=>{
                        log_msg!(LLError,"accept发生错误了：{}",e);
                        return ;
                    },
                    Ok((stream,_))=>{
                        //准备执行握手
                        let mut addr_info: SocketAddrInfo;
                        match stream.peer_addr(){
                            Ok(addr)=>{
                                addr_info = SocketAddrInfo{
                                    ip_v6: false,
                                    port: addr.port(),
                                    ip: [0u8;16],
                                };
                                match addr.ip(){
                                    IpAddr::V4(ref ip)=>{
                                        addr_info.ip[..4].copy_from_slice(ip.octets().as_slice());
                                    },
                                    IpAddr::V6(ref ip)=>{
                                        addr_info.ip.copy_from_slice(ip.octets().as_slice());
                                    }
                                }
                                if let Some(before_handle_shake) = callback.before_handle_shake{
                                    if !before_handle_shake(&addr_info as *const SocketAddrInfo,callback.manager){
                                        //连接刚刚连接上来的时候判定，是否接收
                                        continue;
                                    }
                                }
                            },
                            Err(e)=>{
                                log_msg!(LogLevel::LLError,"获取远程地址的时候发生了错误：{}",e);
                                return;
                            }
                        }
                        //可以继续执行下一步了
                        if let Some(ref acceptor) = acceptor{
                            match acceptor.clone().accept(stream).await{
                                Ok(stream)=>{
                                    tokio::spawn(handle_accept(stream,callback.clone(),tx.subscribe(),addr_info));
                                },
                                Err(e)=>{
                                    log_msg!(LogLevel::LLError,"{} tls握手失败：{}",addr_info,e);
                                    return;
                                }
                            }
                        }else{
                            tokio::spawn(handle_accept(stream,callback.clone(),tx.subscribe(),addr_info));
                        }
                    }
                }
            }

        }
    }
}

async fn handle_accept<S: AsyncRead + AsyncWrite + Unpin>(stream: S,callback: Arc<ServerCallBack>,rx: tokio::sync::watch::Receiver<Option<()>>,addr_info: SocketAddrInfo){
    let handle_ok = std::sync::atomic::AtomicBool::new(true);
    let handle_ok = Arc::new(handle_ok);
    let handle_switch = handle_ok.clone();
    let on_connected: WebSocketConnectedCallBack;
    match callback.on_connected{
        Some(call)=>on_connected=call,
        None=>return,
    }
    match tokio_tungstenite::accept_hdr_async(stream,|req: &HandleShakeRequest<()>, mut res: HandleShakeResponse|->Result<HandleShakeResponse, ErrorResponse>{
        let point_res = &res as *const HandleShakeResponse as usize;
        if let Some(handle_shake) = callback.on_handle_shake{
            if !handle_shake(req as *const HandleShakeRequest<()> as usize,point_res,callback.manager){
                //失败
                handle_switch.store(false,Ordering::SeqCst);
                let has_set_status = {
                    match res.status().as_u16(){
                        100..=299=>false,
                        _=>true,
                    }
                };
                if !has_set_status{
                    *res.status_mut() = StatusCode::NOT_ACCEPTABLE;
                }
            }
        }
        Ok(res)
    }).await{
        Ok(ws_stream)=>{
            {
                if !handle_ok.load(Ordering::SeqCst){
                    log_msg!(LogLevel::LLError,"握手失败了");
                    on_connected(false,0,&addr_info as *const SocketAddrInfo, callback.manager);
                    return ;
                }
            }
            let (msg_sender, msg_recv) = tokio::sync::mpsc::unbounded_channel::<WebSocketMsg>();
            let new_sender = Box::new(msg_sender.clone());
            let new_sender = Box::into_raw(new_sender) as usize;
            let client = on_connected(true,new_sender,&addr_info as *const SocketAddrInfo, callback.manager);
            if client == 0{
                log_msg!(LogLevel::LLDebug,"未成功创建client");
                //释放掉
                unsafe {
                    let _ = Box::from_raw(new_sender as *mut UnboundedSender<WebSocketMsg>);
                }
                return;
            }
            handle_websocket(client,ws_stream,msg_sender,msg_recv,rx,callback).await;
        },
        Err(e)=>{
            on_connected(false,0,&addr_info as *const SocketAddrInfo, callback.manager);
            log_msg!(LLError,"websocket握手{}失败：{}",addr_info.to_string(),e);
        }
    }
}

async fn handle_websocket<S: AsyncRead + AsyncWrite + Unpin>(client: usize,ws_stream: WebSocketStream<S>,msg_sender: UnboundedSender<WebSocketMsg>,mut msg_recv: UnboundedReceiver<WebSocketMsg>,
                    mut rx: tokio::sync::watch::Receiver<Option<()>>,callback: Arc<ServerCallBack>){
    let (mut ws_sender,mut ws_receiver) = ws_stream.split();
    loop {
        tokio::select! {
            _=rx.changed()=>{
                log_msg!(LogLevel::LLDebug,"server退出");
                if let Some(on_close) = callback.on_closed{
                    on_close(1000,StringData::empty(),client,callback.manager);
                }
                let _ =msg_sender.closed().await;
                //将未接收的都接收了，然后回复发送是否成功
                if let Some(on_send) = callback.on_send{
                    loop{
                        if let Ok(send_recv) = msg_recv.try_recv(){
                            //KO掉
                            on_send(false,send_recv,client,callback.manager);
                        }else{
                            break;
                        }
                    }
                }
            },
            _= runtime_quit_notify()=>{
                //runTime退出
                if let Some(on_close) = callback.on_closed{
                    on_close(1000,StringData::empty(),client,callback.manager);
                }
                let _ =msg_sender.closed().await;
                return;
            },
            send_recv = msg_recv.recv()=>{
                //接收到要发送的数据了
                if let Some(mut orgin_msg) = send_recv{
                    match callback.on_send {
                        Some(on_send)=>{
                            if let (Some(msg),vdata) = orgin_msg.message(true){
                                let mut after_send = |send_ok: bool,vdata: Option<Vec<u8>>|{
                                    if let Some(vdata) = vdata{
                                        orgin_msg.data.data = vdata.as_slice().as_ptr() as usize;
                                        orgin_msg.data.len = vdata.len();
                                        //log_msg!(LLDebug,"{:?}",vdata);
                                        on_send(send_ok,orgin_msg,client,callback.manager);
                                    }else{
                                        orgin_msg.data.data = 0;
                                        orgin_msg.data.len = 0;
                                        on_send(send_ok,orgin_msg,client,callback.manager);
                                    }
                                };
                                if let Err(e) = ws_sender.send(msg).await{
                                    log_msg!(LLError,"发送信息错误：{}",e);
                                    after_send(false,vdata);
                                    if let Some(on_close) = callback.on_closed{
                                        on_close(u16::from(CloseCode::Error),StringData::empty(),client,callback.manager);
                                    }
                                    return;
                                }
                                after_send(true,vdata);
                                if let MsgType::Close = MsgType::from(orgin_msg.msg_type){
                                    if let Some(on_close) = callback.on_closed{
                                        on_close(1002,StringData::empty(),client,callback.manager);
                                    }
                                    let _ =msg_sender.closed().await;
                                    return;
                                }
                            }else{
                                //错误了
                                return;
                            }
                        },
                        _=>{
                            if let (Some(msg),_) = orgin_msg.message(false){
                                if let Err(e) = ws_sender.send(msg).await{
                                    //发送失败了
                                    //log_msg(&format!())
                                    log_msg!(LLError,"发送信息发生错误：{}",e);
                                    if let Some(on_close) = callback.on_closed{
                                        on_close(u16::from(CloseCode::Error),StringData::empty(),client,callback.manager);
                                    }
                                    return;
                                }
                                if let MsgType::Close = MsgType::from(orgin_msg.msg_type){
                                    if let Some(on_close) = callback.on_closed{
                                        on_close(1002,StringData::empty(),client,callback.manager);
                                    }
                                    let _ =msg_sender.closed().await;
                                    return;
                                }
                            }else{
                                //发生错误了
                                return;
                            }
                        }
                    }
                }else{
                    //关闭了
                    return;
                }
            },
            recv = ws_receiver.next()=>{
                if let Some(msg) = recv{
                    match msg{
                        Ok(msg)=>{
                            if let tungstenite::Message::Close(frame) = msg{
                                if let Some(on_close) = callback.on_closed{
                                    if let Some(frame) = frame{
                                        let mut reason = StringData::empty();
                                        reason.len = frame.reason.len();
                                        if reason.len > 0{
                                            reason.data = frame.reason.as_ptr() as usize;
                                        }
                                        on_close(u16::from(frame.code),reason,client,callback.manager);
                                    }
                                }
                                let _ = ws_sender.close().await;
                                return ;
                            }else if let crate::MsgType::Close = crate::process_msg(client,callback.manager,msg,callback.on_recv){
                                let _ = ws_sender.close().await;
                                return;
                            }
                        },
                        Err(e)=>{
                            //发生错误，退出
                            if let Some(on_closed) = callback.on_closed{
                                let mut err = format!("recv error:{}",e);
                                on_closed(1011,StringData{
                                    data: err.as_mut_ptr() as usize,
                                    len: err.len(),
                                },client,callback.manager);
                            }
                            return;
                        }
                    }
                }else{
                    //没有接收到数据了
                    return;
                }
            }
        }
    }

}

#[no_mangle]
pub extern "stdcall" fn stop_listen(notify_handle: usize){
    if notify_handle == 0{
        return;
    }
    log_msg!(LogLevel::LLDebug,"stop_listen");
    unsafe {
        let sender = Box::from_raw(notify_handle  as *mut Arc<tokio::sync::watch::Sender<Option<()>>>);
        let _ =sender.send(Some(()));
    }
}
