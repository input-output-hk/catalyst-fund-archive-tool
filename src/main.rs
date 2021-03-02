use chain_addr::Discrimination;
use chain_impl_mockchain::{
    block::Block, chaintypes::HeaderId, fragment::Fragment, transaction::InputEnum,
};
use chain_ser::mempack::{ReadBuf, Readable};
use jormungandr_lib::interfaces::{AccountIdentifier, Address};
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use structopt::StructOpt;

const MAIN_TAG: &str = "HEAD";

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Command {
    /// The path to the Jormungandr database to dump transactions from.
    jormungandr_database: PathBuf,
    /// CSV output directory
    output_dir: PathBuf,
}

#[derive(Serialize)]
struct Vote {
    fragment_id: String,
    caster: Address,
    proposal: u8,
    time: String,
    choice: u8,
    raw_fragment: String,
}

fn main() {
    let command = Command::from_args();

    let db = chain_storage::BlockStore::file(
        command.jormungandr_database.clone(),
        HeaderId::zero_hash()
            .as_bytes()
            .to_owned()
            .into_boxed_slice(),
    )
    .unwrap();

    let tip_id = db.get_tag(MAIN_TAG).unwrap().unwrap();
    let distance = db.get_block_info(tip_id.as_ref()).unwrap().chain_length();

    let mut vote_plan_files = HashMap::new();

    let block_iter = db.iter(tip_id.as_ref(), distance).unwrap();

    for iter_res in block_iter {
        let block_bin = iter_res.unwrap();
        let mut buf = ReadBuf::from(block_bin.as_ref());
        let block: Block = Readable::read(&mut buf).unwrap();

        for fragment in block.fragments() {
            if let Fragment::VoteCast(tx) = fragment {
                let fragment_id = fragment.hash();

                let input = tx.as_slice().inputs().iter().next().unwrap().to_enum();
                let caster = if let InputEnum::AccountInput(account_id, _value) = input {
                    AccountIdentifier::from(account_id)
                        .into_address(Discrimination::Production, "ca")
                } else {
                    panic!();
                };
                let certificate = tx.as_slice().payload().into_payload();

                let writer = vote_plan_files
                    .entry(certificate.vote_plan().clone())
                    .or_insert_with(|| {
                        let mut path = command.output_dir.clone();
                        path.push(format!("vote_plan_{}.csv", certificate.vote_plan()));
                        let file = std::fs::File::create(path).unwrap();
                        csv::Writer::from_writer(file)
                    });

                let choice = match certificate.payload() {
                    chain_impl_mockchain::vote::Payload::Public { choice } => choice.as_byte(),
                    chain_impl_mockchain::vote::Payload::Private { .. } => {
                        unimplemented!("private votes are not supported yet")
                    }
                };

                writer
                    .serialize(Vote {
                        fragment_id: fragment_id.to_string(),
                        caster,
                        proposal: certificate.proposal_index(),
                        time: block.header.block_date().to_string(),
                        raw_fragment: hex::encode(tx.as_ref()),
                        choice,
                    })
                    .unwrap();
            }
        }
    }
}
