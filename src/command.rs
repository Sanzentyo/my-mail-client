use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SendCommand {
    pub command: String,
    pub user_name: String,
    pub timestamp: i64,
    pub args: Args,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Args {
    SendMsg(SendMsgArgs), // "send_msg"コマンド
    CheckMsg(CheckMsgArgs), // "check_msg"コマンド
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMsgArgs {
    pub to: String,
    pub content: String,
    pub connected_id: i64, // ない場合は-1
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMsgResponse {
    pub status: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckMsgArgs {
    pub max_msg: i64, // デフォルトは-1で無制限
    pub recursive: i64, // デフォルトは-1で無制限
    pub from_user_name: String, // 特定の相手からメッセージだけ表示、空文字なら全員表示
    pub since: i64, // タイムスタンプと同じ形式で、-1がデフォルトで指定なし
    pub until: i64, // タイムスタンプと同じ形式で、-1がデフォルトで指定なし
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckMsgResponse {
    pub status: String,
    pub timestamp: i64,
    pub msg: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    pub uuid: i64,
    pub children_msg: Vec<Message>,
}