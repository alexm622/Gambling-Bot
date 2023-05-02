//poker

use redis::RedisError;
use serenity::model::prelude::{ChannelId, UserId, GuildId};
use tracing::info;

use crate::{
    sql::structs::PokerHand,
    utils::{
        deck::{card_to_int, int_to_card},
        poker::get_new_poker_hand,
    },
};

use super::{get_conn, list_contains};

pub async fn get_user_hand( gid: GuildId, cid: ChannelId, uid: UserId) -> Result<PokerHand, RedisError> {
    let mut conn = get_conn().await?;

    info!("attemptng to get user {} hand", uid);
    let key_name = format!("poker_{}_{}_{}",gid, cid, uid);

    //check if empty
    let len = redis::cmd("LLEN")
        .arg(key_name.clone())
        .query::<u8>(&mut conn)?;
    info!("len is {}", len);

    if len == 0 {
        info!("dealing new cards");
        let hand = get_new_poker_hand(gid, cid).await?;
        push_poker_hand(hand,gid, cid, uid).await?;
        return Ok(hand);
    }

    let mut hand_primative = match redis::cmd("LRANGE")
        .arg(key_name)
        .arg(0)
        .arg(len)
        .query::<Vec<u8>>(&mut conn)
    {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let hand: PokerHand = PokerHand {
        one: int_to_card(hand_primative.pop().unwrap()),
        two: int_to_card(hand_primative.pop().unwrap()),
        three: int_to_card(hand_primative.pop().unwrap()),
        four: int_to_card(hand_primative.pop().unwrap()),
        five: int_to_card(hand_primative.pop().unwrap()),
    };

    
    Ok(hand)
}

pub async fn push_poker_hand(
    hand: PokerHand,
    gid: GuildId,
    cid: ChannelId,
    uid: UserId,
) -> Result<(), RedisError> {
    let mut conn = match get_conn().await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let key_name = format!("poker_{}_{}_{}", gid, cid, uid);

    match redis::cmd("DEL")
        .arg(key_name.clone())
        .query::<()>(&mut conn)
    {
        Ok(_) => {}
        Err(e) => return Err(e),
    };

    match redis::cmd("LPUSH")
        .arg(key_name)
        .arg(card_to_int(hand.one))
        .arg(card_to_int(hand.two))
        .arg(card_to_int(hand.three))
        .arg(card_to_int(hand.four))
        .arg(card_to_int(hand.five))
        .query::<()>(&mut conn)
    {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn can_user_discard(uid: UserId, gid: GuildId, cid: ChannelId) -> Result<bool, RedisError> {
    let key_name = format!("poker_candiscard_{}_{}",gid, cid);

    Ok(list_contains(key_name, uid.to_string()).await?)
}

pub async fn activate_can_discard(gid: GuildId,cid: ChannelId) -> Result<(), RedisError> {
    let mut conn = get_conn().await?;

    let key_name = format!("poker_candiscard_{}_{}",gid, cid);

    redis::cmd("DEL").arg(key_name.clone()).query(&mut conn)?;

    let uids = hand_keys_to_uid(get_open_hands(gid,cid).await?)?;

    let mut uid_str = String::new();

    for uid in uids {
        uid_str = format!("{} {}", uid_str, uid);
    }

    redis::cmd("LPUSH")
        .arg(key_name)
        .arg(uid_str)
        .query(&mut conn)?;

    Ok(())
}

pub async fn get_open_hands(gid:GuildId, cid: ChannelId) -> Result<Vec<String>, RedisError> {
    let mut conn = get_conn().await?;

    let pattern = format!("poker_{}_{}_*", gid, cid);

    Ok(redis::cmd("KEYS")
        .arg(pattern)
        .query::<Vec<String>>(&mut conn)?)
}

pub fn hand_keys_to_uid(hand: Vec<String>) -> Result<Vec<UserId>, RedisError> {
    let mut uids: Vec<UserId> = Vec::new();
    hand.clone().into_iter().for_each(|key| {
        let split: Vec<&str> = key.split("_").collect();
        uids.push(UserId::from(
            split.get(1).unwrap().to_string().parse::<u64>().unwrap(),
        ));
    });

    Ok(uids)
}

pub async fn set_can_join(gid: GuildId, cid: ChannelId) -> Result<(), RedisError> {
    let mut conn = get_conn().await?;

    let key = format!("poker_joinable_{}_{}",gid, cid);

    Ok(redis::cmd("SET").arg(key).arg(1).query(&mut conn)?)
}

pub async fn join(gid: GuildId,cid: ChannelId, uid: UserId) -> Result<(), RedisError> {
    let mut conn = get_conn().await?;

    let key = format!("poker_joinned_{}_{}",gid, cid);

    if list_contains(key.clone(), uid.to_string()).await? {
        return Ok(());
    }

    Ok(redis::cmd("LPUSH").arg(key).arg(uid.0).query(&mut conn)?)
}

pub async fn is_joinned(uid: UserId,gid: GuildId, cid: ChannelId) -> Result<bool, RedisError> {
    let key = format!("poker_joinned_{}_{}", gid,cid);

    Ok(list_contains(key.clone(), uid.to_string()).await?)
}

pub async fn can_player_join(gid: GuildId,cid: ChannelId) -> Result<bool, RedisError> {
    let mut conn = get_conn().await?;

    let key = format!("poker_joinable_{}_{}", gid, cid);

    Ok(redis::cmd("EXISTS")
        .arg(key)
        .arg(1)
        .query::<bool>(&mut conn)?)
}

pub async fn joinable_close(gid: GuildId,cid: ChannelId) -> Result<(), RedisError> {
    let mut conn = get_conn().await?;

    let key = format!("poker_joinable_{}_{}",gid, cid);

    Ok(redis::cmd("DEL").arg(key).query(&mut conn)?)
}
