use chain_impl_mockchain::{
    block::Block, chaintypes::HeaderId, fragment::Fragment, transaction::InputEnum,
};
use chain_ser::mempack::{ReadBuf, Readable};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};
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
    caster: String,
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
                    account_id
                } else {
                    panic!();
                };
                let certificate = tx.as_slice().payload().into_payload();

                let (writer, counters) = vote_plan_files
                    .entry(certificate.vote_plan().clone())
                    .or_insert_with(|| {
                        let mut path = command.output_dir.clone();
                        path.push(format!("vote_plan_{}.csv", certificate.vote_plan()));
                        let file = std::fs::File::create(path).unwrap();
                        let vote_counters = BTreeMap::new();
                        (csv::Writer::from_writer(file), vote_counters)
                    });

                let choice = match certificate.payload() {
                    chain_impl_mockchain::vote::Payload::Public { choice } => choice.as_byte(),
                    chain_impl_mockchain::vote::Payload::Private { .. } => {
                        unimplemented!("private votes are not supported yet")
                    }
                };

                let counter = counters
                    .entry(certificate.proposal_index())
                    .or_insert_with(BTreeMap::<_, u64>::new);
                let counter_entry = counter.entry(choice).or_default();
                *counter_entry += 1;

                writer
                    .serialize(Vote {
                        fragment_id: fragment_id.to_string(),
                        caster: hex::encode(caster.as_ref()),
                        proposal: certificate.proposal_index(),
                        time: block.header.block_date().to_string(),
                        raw_fragment: hex::encode(tx.as_ref()),
                        choice,
                    })
                    .unwrap();
            }
        }
    }

    for (vote_plan_id, (_, counters)) in vote_plan_files.iter() {
        let path = {
            let mut path = command.output_dir.clone();
            path.push(format!("vote_plan_summary_{}.csv", vote_plan_id));
            path
        };
        let file = std::fs::File::create(path).unwrap();
        let mut writer = csv::Writer::from_writer(file);

        let max_key = counters
            .values()
            .map(|tree| tree.keys())
            .flatten()
            .cloned()
            .fold(0, u8::max);

        let header = {
            let mut header = vec!["proposal".to_string()];
            for i in 0..max_key {
                header.push(i.to_string());
            }
            header
        };
        writer.write_record(header).unwrap();

        for (proposal, counter) in counters.iter() {
            let mut row = vec![String::new(); max_key as usize + 1];
            row[0] = proposal.to_string();
            for (choice, count) in counter.iter() {
                row[*choice as usize] = count.to_string();
            }
            writer.write_record(row).unwrap();
        }
    }
}
