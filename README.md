# wslib
delphi websocket library wrapped by rust

# 说明
## 初始化InitWslib
  函数中，appMainBlock表示是否阻塞主系统的应用，主要是用来启动rust的tokio的异步阻塞运行时函数，使用Application.Run来阻塞，设置为false的时候，则不会，所以，如果要设置为true，请在应用程序开头去掉Application.Run，然后使用InitWslib来代替

## 使用说明
由于使用的websocket库目前发送无法支持内存复用，为了不继续增加不必要的内存复制以及释放操作，发送的内存块，目前只能使用rust的内部分配单元分配，所以实现了两个结构TRustStream和TRustBuffer来获取操作rust的内存块，另外就是只要Send过了这内存块之后，
就不能再继续使用了，由于rust的所有权机制，他直接移动到发送内部了，使用完毕之后，会自己释放，所以，每次发送之后，会自动将内存结构做一个清理。另外就是对于AfterSend的处理，这个同样不需要释放，都是rust内部的结构内存会自动释放，另外就是rust内部有内
存分配器处理，所以其分配使用效率还是不错的。暂时就先这样使用吧。


## 交流QQ群
本群主要提供一些常用工具库的包装的交流
点击链接加入群聊【DelphiWrap】：https://jq.qq.com/?_wv=1027&k=sk5mVYR7
