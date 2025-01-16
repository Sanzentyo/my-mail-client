このプロジェクトは、TCP通信を使ってメールのようなメッセージのやり取りを実現する学習用のアプリケーションです。

## 注意:実際のメールを送受信することはできません
## Note: This application cannot send or receive actual emails

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


## `list_msg`

### リクエスト(argsの内容)
``` json
list_msg {
   "max_msg": int, // デフォルトは-1で無制限
   "from_user_name": string, // 特定の相手からメッセージだけ表示、空文字なら全員表示
   "to_user_name": string, // 特定の相手へのメッセージだけ表示、空文字なら全員表示
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

## `search_msg`
``` json
search_msg {
   "select_uuid": int, // msgのuuid
   "recursive": int, // スレッドを何回まで再帰的に検索するか、デフォルトは0、-1で見つからなくなるまで永遠に
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

# セットアップ

```bash
# リポジトリをクローン
git clone https://github.com/yourusername/my-mail-client.git
cd my-mail-client

# サーバーを起動
cargo run --bin server

# 別のターミナルでクライアントを起動
cargo run --bin client
```

# 使用方法

## メッセージの送信 (send)

1. コマンドプロンプトで `send` を入力
2. 以下の情報を入力:
   - 宛先 (to): 送信したい相手のユーザー名
   - メッセージ内容 (content): 送信したいメッセージ
   - connected_id: 返信する場合は元のメッセージのUUID、新規の場合は空白

## メッセージの一覧表示 (list)

1. コマンドプロンプトで `list` を入力
2. フィルター条件を入力:
   - max_msg: 表示する最大メッセージ数（空白で無制限）
   - from_user_name: 特定の送信者のみ表示
   - to_user_name: 特定の受信者のみ表示
   - since: この時刻以降のメッセージを表示
   - until: この時刻までのメッセージを表示

## スレッド検索 (search)

1. コマンドプロンプトで `search` を入力
2. 検索条件を入力:
   - select_uuid: 検索したいメッセージのUUID
   - recursive: スレッドを何階層まで検索するか（0: 直接の親子のみ、-1: 無制限）

# デモンストレーション

```bash
# ユーザーAがメッセージを送信
> send
宛先 (to) を入力してください: userB
メッセージ内容 (content) を入力してください: こんにちは！
connected_id (なければ空白): 

# ユーザーBが返信
> send
宛先 (to) を入力してください: userA
メッセージ内容 (content) を入力してください: はじめまして！
connected_id (なければ空白): 1

# スレッドを検索
> search
select_uuid: 1
recursive (例: 0、しないなら空白): -1

# 結果表示
|-------------------------------
| From: userA
| To: userB
| Content: こんにちは！
| UUID: 1
| Timestamp: 1707123456
| Children:
  |-----------------------------
  | From: userB
  | To: userA
  | Content: はじめまして！
  | UUID: 2
  | Timestamp: 1707123789
```

# 注意事項

- タイムスタンプはUNIX時間（秒）で扱われます
- UUIDは自動的に割り当てられる連番です
- データベースファイルは `./msg.db` に作成されます