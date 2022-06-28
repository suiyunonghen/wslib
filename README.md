# wslib
delphi websocket library wrapped by rust

# 说明
## 初始化InitWslib
  函数中，appMainBlock表示是否阻塞主系统的应用，主要是用来启动rust的tokio的异步阻塞运行时函数，使用Application.Run来阻塞，设置为false的时候，则不会，所以，如果要设置为true，请在应用程序开头去掉Application.Run，然后使用InitWslib来代替

## 包装提供了两套函数，一套是send，一套是sendcopy，
- send类函数
  send中传递的内存地址，需要自己持有其生命周期，直到AfterSendMsg的时候，才能释放掉，通过这个机制可以使用内存池来做处理
- sendcopy类函数
  使用sendcopy函数的时候，其中的内存不必管，因为此函数会先将数据复制，所以可以在sendcopy之后立即释放，同时AfterSendMsg中的内存信息也是nil。

## 交流QQ群
本群主要提供一些常用工具库的包装的交流
点击链接加入群聊【DelphiWrap】：https://jq.qq.com/?_wv=1027&k=sk5mVYR7
