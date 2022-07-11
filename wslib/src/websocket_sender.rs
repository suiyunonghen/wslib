use futures_util::{SinkExt, stream::SplitSink};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::WebSocketStream;
use crate::{log_msg, LOG, MsgType, WebSocketMsg, StringData, WebSocketSendCallBack, WebSocketCloseCallBack};
use crate::LogLevel::{LLError};

#[no_mangle]
pub extern "stdcall" fn send_data(msg: WebSocketMsg,raw: usize)->bool{
    if let MsgType::Close = MsgType::from(msg.msg_type){
        //释放掉这个writer
        let sender = unsafe{
            Box::from_raw(raw as *mut tokio::sync::mpsc::UnboundedSender<WebSocketMsg>)
        };
        return match sender.send(msg) {
            Ok(()) => true,
            Err(e) => {
                log_msg!(LLError,"发送消息错误：{}", e);
                false
            },
        }
    }
    let sender = unsafe{
        let sender = raw as *mut tokio::sync::mpsc::UnboundedSender<WebSocketMsg>;
        &mut *sender
    };
    match sender.send(msg){
        Ok(())=>true,
        Err(e)=>{
            log_msg!(LLError,"发送消息错误：{}",e);
            false
        },
    }
}

#[no_mangle]
pub extern "stdcall" fn free_writer(raw_writer: usize){
    let _ = unsafe{
        Box::from_raw(raw_writer as *mut tokio::sync::mpsc::UnboundedSender<WebSocketMsg>)
    };
}


pub async fn handle_send<T>(mut msg_to_send: WebSocketMsg, on_send: Option<WebSocketSendCallBack>, on_closed: Option<WebSocketCloseCallBack>, ws_sender: &mut SplitSink<WebSocketStream<T>, Message>, client: usize, manager: usize)->bool
where T: AsyncRead + AsyncWrite + Unpin
{
    let do_close = |msg_type: u16|->bool{
        if let MsgType::Close = MsgType::from(msg_type){
            if let Some(on_close) = on_closed{
                on_close(1002,StringData::empty(),client,manager);
            }
            return true;
        }
        false
    };
    match on_send {
        Some(on_send)=>{
            if let (Some(msg),value_data) = msg_to_send.message(true){
                let mut after_send = |send_ok: bool,value_data: Option<Vec<u8>>|{
                    if let Some(value_data) = value_data{
                        msg_to_send.data.data = value_data.as_slice().as_ptr() as usize;
                        msg_to_send.data.len = value_data.len();
                        on_send(send_ok,msg_to_send,client,manager);
                    }else{
                        msg_to_send.data.data = 0;
                        msg_to_send.data.len = 0;
                        on_send(send_ok,msg_to_send,client,manager);
                    }
                };
                if let Err(e) = ws_sender.send(msg).await{
                    log_msg!(LLError,"发送信息错误：{}",e);
                    after_send(false,value_data);
                    if let Some(on_close) = on_closed{
                        on_close(u16::from(CloseCode::Error),StringData::empty(),client,manager);
                    }
                    return false;
                }
                after_send(true,value_data);
                if do_close(msg_to_send.msg_type){
                    return false
                }
            }else{
                //错误了
                return false;
            }
        },
        _=>{
            if let (Some(msg),_) = msg_to_send.message(false){
                if let Err(e) = ws_sender.send(msg).await{
                    //发送失败了
                    log_msg!(LLError,"发送信息发生错误：{}",e);
                    if let Some(on_close) = on_closed{
                        on_close(u16::from(CloseCode::Error),StringData::empty(),client,manager);
                    }
                    return false;
                }
                if do_close(msg_to_send.msg_type){
                    return false
                }
            }else{
                //发生错误了
                return false;
            }
        }
    }
    true
}