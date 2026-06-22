use anyhow::{bail, Result};
use minicbor::{Decoder, Encoder};

use crate::token::{
    Algorithm, LeaseState, ResourceScope, TokenHeader, TokenPayload, TokenType, TrustAnchor,
    WireToken,
};

fn enc_err<E: std::fmt::Display>(e: E) -> anyhow::Error {
    anyhow::anyhow!("{e}")
}

pub fn encode_signed_body(header: &TokenHeader, payload: &TokenPayload) -> Result<Vec<u8>> {
    let mut enc = Encoder::new(Vec::new());
    enc.map(2).map_err(enc_err)?;
    encode_header(&mut enc, header)?;
    encode_payload(&mut enc, payload)?;
    Ok(enc.into_writer())
}

pub fn encode_wire_token(token: &WireToken) -> Result<Vec<u8>> {
    let mut enc = Encoder::new(Vec::new());
    enc.map(3).map_err(enc_err)?;
    encode_header(&mut enc, &token.header)?;
    encode_payload(&mut enc, &token.payload)?;
    enc.bytes(&token.signature).map_err(enc_err)?;
    Ok(enc.into_writer())
}

pub fn decode_wire_token(bytes: &[u8]) -> Result<WireToken> {
    let mut dec = Decoder::new(bytes);
    let len = dec
        .map()
        .map_err(enc_err)?
        .ok_or_else(|| anyhow::anyhow!("expected map"))?;
    if len != 3 {
        bail!("wire token map must have 3 entries");
    }
    let header = decode_header(&mut dec)?;
    let payload = decode_payload(&mut dec)?;
    let signature = dec.bytes().map_err(enc_err)?.to_vec();
    Ok(WireToken {
        header,
        payload,
        signature,
    })
}

fn encode_header(enc: &mut Encoder<Vec<u8>>, header: &TokenHeader) -> Result<()> {
    enc.str("hdr").map_err(enc_err)?;
    enc.map(4).map_err(enc_err)?;
    enc.u32(1).map_err(enc_err)?.u32(header.ver).map_err(enc_err)?;
    enc.u32(2).map_err(enc_err)?.u32(token_type_to_u32(header.typ)).map_err(enc_err)?;
    enc.u32(3).map_err(enc_err)?.u32(algorithm_to_u32(header.alg)).map_err(enc_err)?;
    enc.u32(4).map_err(enc_err)?.u32(anchor_to_u32(header.anchor)).map_err(enc_err)?;
    Ok(())
}

fn decode_header(dec: &mut Decoder<'_>) -> Result<TokenHeader> {
    let _key = dec.str().map_err(enc_err)?;
    let _len = dec
        .map()
        .map_err(enc_err)?
        .ok_or_else(|| anyhow::anyhow!("hdr map"))?;
    let mut ver = 0u32;
    let mut typ = TokenType::Capability;
    let mut alg = Algorithm::Ed25519;
    let mut anchor = TrustAnchor::None;
    for _ in 0..4 {
        let k = dec.u32().map_err(enc_err)?;
        let v = dec.u32().map_err(enc_err)?;
        match k {
            1 => ver = v,
            2 => typ = u32_to_token_type(v)?,
            3 => alg = u32_to_algorithm(v)?,
            4 => anchor = u32_to_anchor(v)?,
            _ => {}
        }
    }
    Ok(TokenHeader { ver, typ, alg, anchor })
}

fn encode_payload(enc: &mut Encoder<Vec<u8>>, payload: &TokenPayload) -> Result<()> {
    enc.str("pld").map_err(enc_err)?;
    enc.map(9).map_err(enc_err)?;
    enc.u32(10).map_err(enc_err)?.bytes(&payload.iss).map_err(enc_err)?;
    enc.u32(11).map_err(enc_err)?.bytes(&payload.sub).map_err(enc_err)?;
    enc.u32(12).map_err(enc_err)?.bytes(&payload.ctx).map_err(enc_err)?;
    enc.u32(13).map_err(enc_err)?;
    encode_scope(enc, &payload.scope)?;
    enc.u32(14).map_err(enc_err)?.u64(payload.exp).map_err(enc_err)?;
    enc.u32(15).map_err(enc_err)?.u64(payload.nbf).map_err(enc_err)?;
    enc.u32(16).map_err(enc_err)?.u32(payload.uses).map_err(enc_err)?;
    enc.u32(17)
        .map_err(enc_err)?
        .u32(lease_state_to_u32(payload.state))
        .map_err(enc_err)?;
    enc.u32(18).map_err(enc_err)?.bytes(&payload.jti).map_err(enc_err)?;
    Ok(())
}

fn decode_payload(dec: &mut Decoder<'_>) -> Result<TokenPayload> {
    let _key = dec.str().map_err(enc_err)?;
    let _len = dec
        .map()
        .map_err(enc_err)?
        .ok_or_else(|| anyhow::anyhow!("pld map"))?;
    let mut iss = Vec::new();
    let mut sub = Vec::new();
    let mut ctx = Vec::new();
    let mut scope = ResourceScope::Raw(Vec::new());
    let mut exp = 0u64;
    let mut nbf = 0u64;
    let mut uses = 0u32;
    let mut state = LeaseState::Requested;
    let mut jti = Vec::new();
    for _ in 0..9 {
        let k = dec.u32().map_err(enc_err)?;
        match k {
            10 => iss = dec.bytes().map_err(enc_err)?.to_vec(),
            11 => sub = dec.bytes().map_err(enc_err)?.to_vec(),
            12 => ctx = dec.bytes().map_err(enc_err)?.to_vec(),
            13 => scope = decode_scope(dec)?,
            14 => exp = dec.u64().map_err(enc_err)?,
            15 => nbf = dec.u64().map_err(enc_err)?,
            16 => uses = dec.u32().map_err(enc_err)?,
            17 => state = u32_to_lease_state(dec.u32().map_err(enc_err)?)?,
            18 => jti = dec.bytes().map_err(enc_err)?.to_vec(),
            _ => bail!("unknown payload field {k}"),
        }
    }
    Ok(TokenPayload {
        iss,
        sub,
        ctx,
        scope,
        exp,
        nbf,
        uses,
        state,
        jti,
    })
}

fn encode_scope(enc: &mut Encoder<Vec<u8>>, scope: &ResourceScope) -> Result<()> {
    match scope {
        ResourceScope::File(f) => {
            let len = if f.inode.is_some() { 3 } else { 2 };
            enc.map(len).map_err(enc_err)?;
            enc.str("path").map_err(enc_err)?.str(&f.path).map_err(enc_err)?;
            enc.str("access").map_err(enc_err)?.u32(f.access).map_err(enc_err)?;
            if let Some(inode) = f.inode {
                enc.str("inode").map_err(enc_err)?.u64(inode).map_err(enc_err)?;
            }
        }
        ResourceScope::Network(n) => {
            enc.map(4).map_err(enc_err)?;
            enc.str("proto").map_err(enc_err)?.u32(n.proto).map_err(enc_err)?;
            enc.str("dst_ip").map_err(enc_err)?.bytes(&n.dst_ip).map_err(enc_err)?;
            enc.str("dst_port").map_err(enc_err)?.u32(n.dst_port as u32).map_err(enc_err)?;
            enc.str("bytes").map_err(enc_err)?.u64(n.bytes).map_err(enc_err)?;
        }
        ResourceScope::Raw(bytes) => {
            enc.bytes(bytes).map_err(enc_err)?;
        }
    }
    Ok(())
}

fn decode_scope(dec: &mut Decoder<'_>) -> Result<ResourceScope> {
    match dec.datatype().map_err(enc_err)? {
        minicbor::data::Type::Map => {
            let len = dec
                .map()
                .map_err(enc_err)?
                .ok_or_else(|| anyhow::anyhow!("scope map"))?;
            let mut path = String::new();
            let mut access = 0u32;
            let mut inode = None;
            let mut proto = 0u32;
            let mut dst_ip = Vec::new();
            let mut dst_port = 0u16;
            let mut bytes = 0u64;
            for _ in 0..len {
                let key = dec.str().map_err(enc_err)?;
                match key {
                    "path" => path = dec.str().map_err(enc_err)?.to_string(),
                    "access" => access = dec.u32().map_err(enc_err)?,
                    "inode" => inode = Some(dec.u64().map_err(enc_err)?),
                    "proto" => proto = dec.u32().map_err(enc_err)?,
                    "dst_ip" => dst_ip = dec.bytes().map_err(enc_err)?.to_vec(),
                    "dst_port" => dst_port = dec.u32().map_err(enc_err)? as u16,
                    "bytes" => bytes = dec.u64().map_err(enc_err)?,
                    _ => dec.skip().map_err(enc_err)?,
                }
            }
            if !path.is_empty() {
                Ok(ResourceScope::File(crate::token::FileScope {
                    path,
                    access,
                    inode,
                }))
            } else {
                Ok(ResourceScope::Network(crate::token::NetworkScope {
                    proto,
                    dst_ip,
                    dst_port,
                    bytes,
                }))
            }
        }
        _ => Ok(ResourceScope::Raw(dec.bytes().map_err(enc_err)?.to_vec())),
    }
}

fn token_type_to_u32(t: TokenType) -> u32 {
    match t {
        TokenType::Capability => 1,
        TokenType::Lease => 2,
        TokenType::Delegation => 3,
        TokenType::Revocation => 4,
    }
}

fn u32_to_token_type(v: u32) -> Result<TokenType> {
    match v {
        1 => Ok(TokenType::Capability),
        2 => Ok(TokenType::Lease),
        3 => Ok(TokenType::Delegation),
        4 => Ok(TokenType::Revocation),
        other => bail!("unknown token type {other}"),
    }
}

fn algorithm_to_u32(a: Algorithm) -> u32 {
    match a {
        Algorithm::MlDsa87 => 1,
        Algorithm::Ed25519 => 2,
        Algorithm::MlDsa65 => 3,
    }
}

fn u32_to_algorithm(v: u32) -> Result<Algorithm> {
    match v {
        1 => Ok(Algorithm::MlDsa87),
        2 => Ok(Algorithm::Ed25519),
        3 => Ok(Algorithm::MlDsa65),
        other => bail!("unknown algorithm {other}"),
    }
}

fn anchor_to_u32(a: TrustAnchor) -> u32 {
    match a {
        TrustAnchor::None => 0,
        TrustAnchor::UiEvent => 1,
        TrustAnchor::Biometric => 2,
        TrustAnchor::Hardware => 3,
        TrustAnchor::Federated => 4,
    }
}

fn u32_to_anchor(v: u32) -> Result<TrustAnchor> {
    match v {
        0 => Ok(TrustAnchor::None),
        1 => Ok(TrustAnchor::UiEvent),
        2 => Ok(TrustAnchor::Biometric),
        3 => Ok(TrustAnchor::Hardware),
        4 => Ok(TrustAnchor::Federated),
        other => bail!("unknown anchor {other}"),
    }
}

fn lease_state_to_u32(s: LeaseState) -> u32 {
    match s {
        LeaseState::Requested => 0,
        LeaseState::Granted => 1,
        LeaseState::Renewing => 2,
        LeaseState::Expired => 3,
        LeaseState::Revoked => 4,
        LeaseState::Suspended => 5,
    }
}

fn u32_to_lease_state(v: u32) -> Result<LeaseState> {
    match v {
        0 => Ok(LeaseState::Requested),
        1 => Ok(LeaseState::Granted),
        2 => Ok(LeaseState::Renewing),
        3 => Ok(LeaseState::Expired),
        4 => Ok(LeaseState::Revoked),
        5 => Ok(LeaseState::Suspended),
        other => bail!("unknown lease state {other}"),
    }
}

