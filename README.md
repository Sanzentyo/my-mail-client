# 

# プロトコル
- テキストかバイナリか: テキスト(JSON)

- intは全部64bitにする

## 送信
``` json
{
   "command": string,
   "user_name": string,
   "timestamp": int,
   "args": args
}
```

## args

## `send_msg` 

### リクエスト(argsの内容)

``` json
send_msg {
   "to": [string],
   "content": string,
   "connected_id": int // ない場合は-1
}
```


### レスポンス
``` json
responce {
   "status": string,
   "timestamp": int
}
```


## `check_msg`

### リクエスト(argsの内容)
``` json
check_msg {
   "max_msg": int, // デフォルトは-1で無制限
   "recursive": int, // スレッドを何回まで再帰的に検索するか、デフォルトは0
   "from_user_name": string, // 特定の相手からメッセージだけ表示、空文字なら全員表示
   "since": int, // タイムスタンプと同じ形式で、-1がデフォルトで指定なし
   "until": int, // タイムスタンプと同じ形式で、-1がデフォルトで指定なし
}
```

### レスポンス

``` json
responce {
   "status": string,
   "timestamp": int,
   "msg": [msg]
}
```

``` json
msg {
   "from": string,
   "to": string,
   "content": string,
   "timestamp": int,
   "uuid": int,
   "children_msg": [msg]
}
```


## DBに保存されるmsgの内容
* from
* to
* content
* uuid
* connected-msg-uuid
*  timestamp