use std::net::IpAddr;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use crate::{log_msg, LOG, StringData, WebSocketCloseCallBack, WebSocketConnectedCallBack, WebSocketMsg, WebSocketRecvCallBack, WebSocketSendCallBack, SocketAddrInfo, websocket_sender};
use tokio_tungstenite::tungstenite::{self,client::IntoClientRequest,handshake::client::Request};
use crate::tungstenite::http::header::HeaderName;

pub type StringDataArray = StringData; //data中是StringData指针，len是多少个StringData

#[repr(C)]
pub struct HeaderValue{
    header: StringData,
    value: StringData,
}

pub type HeaderValues = StringData; //data中是HeaderValue指针

#[repr(C)]
pub struct ClientConfig{
    pub ws_url: StringData,
    pub protocol: StringDataArray,
    pub extensions: StringDataArray,
    pub custom_headers: HeaderValues,
}

impl IntoClientRequest for ClientConfig {
    fn into_client_request(self) -> tungstenite::Result<Request> {
        let ws_url = unsafe{
            String::from_raw_parts(self.ws_url.data as *mut u8,self.ws_url.len,self.ws_url.len)
        };
        let ws_url = std::mem::ManuallyDrop::new(ws_url);
        let mut request = (&*ws_url).into_client_request()?;
        let headers = request.headers_mut();
        let sz = std::mem::size_of::<StringData>();
        log_msg!(crate::LogLevel::LLDebug,"StringDataSize={}",sz);
        if self.protocol.data != 0 && self.protocol.len != 0{
            let mut cur_proto_data = self.protocol.data;
            let mut proto_s = String::with_capacity(128);
            for i in 0..self.protocol.len{
                let cur_proto = cur_proto_data  as *const StringData;
                let s = unsafe{
                    String::from_raw_parts((*cur_proto).data as *mut u8,(*cur_proto).len,(*cur_proto).len)
                };
                log_msg!(crate::LogLevel::LLDebug,"protocol {}={},len={}",i,&s,s.len());
                let s = std::mem::ManuallyDrop::new(s);
                if i != 0{
                    proto_s.push(',');
                }
                proto_s.push_str(&s);
                cur_proto_data = cur_proto_data + sz;
            }
            if proto_s.len() != 0{
                headers.insert("Sec-WebSocket-Protocol",proto_s.parse().unwrap());
            }
        }
        if self.extensions.data != 0 && self.extensions.len != 0{
            //增加websocket的扩展协议
            let mut cur_ext_data = self.extensions.data;
            let mut extensions = String::with_capacity(128);
            for i in 0..self.extensions.len{
                let cur_extension = cur_ext_data  as *const StringData;
                let s = unsafe{
                    String::from_raw_parts((*cur_extension).data as *mut u8,(*cur_extension).len,(*cur_extension).len)
                };
                let s = std::mem::ManuallyDrop::new(s);
                if i != 0{
                    extensions.push(',');
                }
                extensions.push_str(&s);
                cur_ext_data = cur_ext_data + sz;
            }
            if extensions.len() != 0{
                headers.insert("Sec-WebSocket-Extensions",extensions.parse().unwrap());
            }
        }
        if self.custom_headers.data != 0 && self.custom_headers.len != 0{
            let mut head_data = self.custom_headers.data;
            let sz = std::mem::size_of::<HeaderValue>();
            log_msg!(crate::LogLevel::LLDebug,"HeaderValueSize={}",sz);
            for _i in 0..self.custom_headers.len{
                let cur_head = head_data  as *const HeaderValue;
                unsafe {
                    let head: Vec<u8> = Vec::from_raw_parts((*cur_head).header.data as *mut u8,(*cur_head).header.len,(*cur_head).header.len);
                    let v = String::from_raw_parts((*cur_head).value.data as *mut u8,(*cur_head).value.len,(*cur_head).value.len);
                    log_msg!(crate::LogLevel::LLDebug,"value={},len={}",&v,v.len());
                    let head = std::mem::ManuallyDrop::new(head);
                    let v = std::mem::ManuallyDrop::new(v);
                    headers.insert(HeaderName::from_bytes(head.as_slice()).unwrap(),v.parse().unwrap());
                }
                head_data = head_data + sz;
            }
        }
        Ok(request)
    }
}

#[repr(C)]
pub struct ClientCallBack{
    pub manager: usize,
    pub on_connected: Option<WebSocketConnectedCallBack>,
    pub on_recv: Option<WebSocketRecvCallBack>,
    pub on_send: Option<WebSocketSendCallBack>,
    pub on_closed: Option<WebSocketCloseCallBack>,
}

#[no_mangle]
pub extern "stdcall" fn ws_connect(cfg: ClientConfig,callback: ClientCallBack){
    let on_connected: WebSocketConnectedCallBack;
    match callback.on_connected{
        Some(call)=>on_connected=call,
        None=>{
            log_msg!(crate::LogLevel::LLError,"必须指定on_connected");
            return;
        },
    }
    log_msg!(crate::LogLevel::LLDebug,"ws_connect Start");
    let (msg_sender, mut msg_recv) = tokio::sync::mpsc::unbounded_channel::<WebSocketMsg>();
    let new_sender = Box::new(msg_sender.clone());
    let new_sender = Box::into_raw(new_sender) as usize;
    tokio::spawn(async move{
        match tokio_tungstenite::connect_async(cfg).await{
            Ok((stream,_))=>{
                let addr = match stream.get_ref() {
                    tokio_tungstenite::MaybeTlsStream::<TcpStream>::Plain(ref s)=>{
                        log_msg!(crate::LogLevel::LLDebug,"连接成功，准备接入了");
                        s.peer_addr()
                    },
                    tokio_tungstenite::MaybeTlsStream::<TcpStream>::Rustls(ref s)=>{
                        let (tcp,_) = s.get_ref();
                        tcp.peer_addr()
                    }
                    _ => {
                        return;
                    },
                };

                let mut addr_info: SocketAddrInfo;
                match addr {
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
                    },
                    Err(e)=>{
                        log_msg!(crate::LogLevel::LLError,"获取远程地址的时候发生了错误：{}",e);
                        return;
                    }
                }
                log_msg!(crate::LogLevel::LLDebug,"准备接入了");

                let client = on_connected(true,new_sender,&addr_info as *const SocketAddrInfo, callback.manager);
                if client == 0{
                    log_msg!(crate::LogLevel::LLDebug,"未成功创建client");
                    //释放掉
                    unsafe {
                        let _ = Box::from_raw(new_sender as *mut UnboundedSender<WebSocketMsg>);
                    }
                    return;
                }
                let (mut ws_sender, mut ws_receiver) = stream.split();
                loop {
                    tokio::select! {
                        _= crate::runtime_quit_notify()=>{
                            //退出了程序
                            if let Some(on_close) = callback.on_closed{
                                on_close(1000,StringData::empty(),client,callback.manager);
                            }
                            return ;
                        },
                        send_recv = msg_recv.recv()=>{
                            //接收到要发送的数据了
                            if let Some(orgin_msg) = send_recv{
                                if !websocket_sender::handle_send(orgin_msg,callback.on_send,callback.on_closed,&mut ws_sender,client,callback.manager).await{
                                    let _ = msg_sender.closed().await;
                                    return;
                                }
                            }else{
                                //关闭了
                                return;
                            }
                        },
                        recv = ws_receiver.next()=>{
                            //接收到消息
                            if let Some(msg) = recv{
                                if !crate::handle_recv(msg,callback.on_closed,callback.on_recv,&mut ws_sender,client,callback.manager).await{
                                    let _ = msg_sender.closed().await;
                                    return;
                                }
                            }else{
                                //没有接收到数据了
                                return;
                            }
                        }
                    }
                }
            },
            Err(e)=>{
                on_connected(false,0,0 as *const SocketAddrInfo, callback.manager);
                log_msg!(crate::LogLevel::LLError,"websocket服务连接失败：{}",e);
            }
        }
    });
}