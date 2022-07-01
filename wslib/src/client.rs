use crate::{StringData, WebSocketCloseCallBack, WebSocketConnectedCallBack, WebSocketRecvCallBack, WebSocketSendCallBack};
use tokio_tungstenite::tungstenite::{self,client::IntoClientRequest,handshake::client::Request};

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
        if self.protocol.data != 0 && self.protocol.len != 0{
            let mut cur_proto_data = self.protocol.data;
            let sz = std::mem::size_of::<usize>();
            let mut protos = String::with_capacity(128);
            for i in 0..self.protocol.len{
                let cur_proto = cur_proto_data  as *const StringData;
                let s = unsafe{
                    String::from_raw_parts((*cur_proto).data as *mut u8,(*cur_proto).len,(*cur_proto).len)
                };
                let s = std::mem::ManuallyDrop::new(s);
                if i != 0{
                    protos.push(',');
                }
                protos.push_str(&s);
                cur_proto_data = cur_proto_data + sz;
            }
            if protos.len() != 0{
                headers.insert("Sec-WebSocket-Protocol",protos.parse().unwrap());
            }
        }
        //request.headers_mut().insert()
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


pub extern "stdcall" fn connect(cfg: ClientConfig,callback: ClientCallBack){
    //tokio_tungstenite::connect_async(cfg)
}