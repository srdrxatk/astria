use std::time::Duration;

use astria_conductor::config::CommitLevel;
use futures::future::{
    join,
    join4,
};
use tokio::time::timeout;

use crate::{
    helpers::spawn_conductor,
    mount_celestia_blobs,
    mount_celestia_header_network_head,
    mount_executed_block,
    mount_get_commitment_state,
    mount_get_genesis_info,
    mount_sequencer_commit,
    mount_sequencer_genesis,
    mount_sequencer_validator_set,
    mount_update_commitment_state,
};

#[tokio::test]
async fn simple() {
    let test_conductor = spawn_conductor(CommitLevel::FirmOnly).await;

    mount_get_genesis_info!(
        test_conductor,
        sequencer_genesis_block_height: 1,
        celestia_base_block_height: 1,
        celestia_block_variance: 10,
    );

    mount_get_commitment_state!(
        test_conductor,
        firm: (
            number: 1,
            hash: [1; 64],
            parent: [0; 64],
        ),
        soft: (
            number: 1,
            hash: [1; 64],
            parent: [0; 64],
        ),
    );

    mount_sequencer_genesis!(test_conductor);

    mount_celestia_header_network_head!(
        test_conductor,
        height: 1u32,
    );

    mount_celestia_blobs!(
        test_conductor,
        celestia_height: 1,
        sequencer_height: 3,
    );

    mount_sequencer_commit!(
        test_conductor,
        height: 3u32,
    );

    mount_sequencer_validator_set!(test_conductor, height: 2u32);

    let execute_block = mount_executed_block!(
        test_conductor,
        number: 2,
        hash: [2; 64],
        parent: [1; 64],
    );

    let update_commitment_state = mount_update_commitment_state!(
        test_conductor,
        firm: (
            number: 2,
            hash: [2; 64],
            parent: [1; 64],
        ),
        soft: (
            number: 2,
            hash: [2; 64],
            parent: [1; 64],
        ),
    );

    timeout(
        Duration::from_millis(2000),
        join(
            execute_block.wait_until_satisfied(),
            update_commitment_state.wait_until_satisfied(),
        ),
    )
    .await
    .expect(
        "conductor should have executed the firm block and updated the firm commitment state \
         within 1000ms",
    );
}

#[tokio::test]
async fn submits_two_heights_in_sucession() {
    let test_conductor = spawn_conductor(CommitLevel::FirmOnly).await;

    mount_get_genesis_info!(
        test_conductor,
        sequencer_genesis_block_height: 1,
        celestia_base_block_height: 1,
        celestia_block_variance: 10,
    );

    mount_get_commitment_state!(
        test_conductor,
        firm: (
            number: 1,
            hash: [1; 64],
            parent: [0; 64],
        ),
        soft: (
            number: 1,
            hash: [1; 64],
            parent: [0; 64],
        ),
    );

    mount_sequencer_genesis!(test_conductor);

    mount_celestia_header_network_head!(
        test_conductor,
        height: 2u32,
    );

    mount_celestia_blobs!(
        test_conductor,
        celestia_height: 1,
        sequencer_height: 3,
    );

    mount_celestia_blobs!(
        test_conductor,
        celestia_height: 2,
        sequencer_height: 4,
    );

    mount_sequencer_commit!(
        test_conductor,
        height: 3u32,
    );

    mount_sequencer_validator_set!(test_conductor, height: 2u32);

    mount_sequencer_commit!(
        test_conductor,
        height: 4u32,
    );

    mount_sequencer_validator_set!(test_conductor, height: 3u32);

    let execute_block_number_2 = mount_executed_block!(
        test_conductor,
        number: 2,
        hash: [2; 64],
        parent: [1; 64],
    );

    let update_commitment_state_number_2 = mount_update_commitment_state!(
        test_conductor,
        firm: (
            number: 2,
            hash: [2; 64],
            parent: [1; 64],
        ),
        soft: (
            number: 2,
            hash: [2; 64],
            parent: [1; 64],
        ),
    );

    let execute_block_number_3 = mount_executed_block!(
        test_conductor,
        number: 3,
        hash: [3; 64],
        parent: [2; 64],
    );

    let update_commitment_state_number_3 = mount_update_commitment_state!(
        test_conductor,
        firm: (
            number: 3,
            hash: [3; 64],
            parent: [2; 64],
        ),
        soft: (
            number: 3,
            hash: [3; 64],
            parent: [2; 64],
        ),
    );

    timeout(
        Duration::from_millis(2000),
        join4(
            execute_block_number_2.wait_until_satisfied(),
            update_commitment_state_number_2.wait_until_satisfied(),
            execute_block_number_3.wait_until_satisfied(),
            update_commitment_state_number_3.wait_until_satisfied(),
        ),
    )
    .await
    .expect(
        "conductor should have executed the soft block and updated the soft commitment state \
         within 2000ms",
    );
}

#[tokio::test]
async fn skips_already_executed_heights() {
    let test_conductor = spawn_conductor(CommitLevel::FirmOnly).await;

    mount_get_genesis_info!(
        test_conductor,
        sequencer_genesis_block_height: 1,
        celestia_base_block_height: 1,
        celestia_block_variance: 10,
    );

    mount_get_commitment_state!(
        test_conductor,
        firm: (
            number: 5,
            hash: [1; 64],
            parent: [0; 64],
        ),
        soft: (
            number: 5,
            hash: [1; 64],
            parent: [0; 64],
        ),
    );
    mount_sequencer_genesis!(test_conductor);

    mount_celestia_header_network_head!(
        test_conductor,
        height: 2u32,
    );

    // The blob with sequencer height 5 will be fetched but never forwarded
    mount_celestia_blobs!(
        test_conductor,
        celestia_height: 1,
        sequencer_height: 6,
    );

    mount_celestia_blobs!(
        test_conductor,
        celestia_height: 2,
        sequencer_height: 7,
    );

    mount_sequencer_commit!(
        test_conductor,
        height: 6u32,
    );

    mount_sequencer_validator_set!(test_conductor, height: 5u32);

    mount_sequencer_commit!(
        test_conductor,
        height: 7u32,
    );

    mount_sequencer_validator_set!(test_conductor, height: 6u32);

    let execute_block = mount_executed_block!(
        test_conductor,
        number: 6,
        hash: [2; 64],
        parent: [1; 64],
    );

    let update_commitment_state = mount_update_commitment_state!(
        test_conductor,
        firm: (
            number: 6,
            hash: [2; 64],
            parent: [1; 64],
        ),
        soft: (
            number: 6,
            hash: [2; 64],
            parent: [1; 64],
        ),
    );

    timeout(
        Duration::from_millis(1000),
        join(
            execute_block.wait_until_satisfied(),
            update_commitment_state.wait_until_satisfied(),
        ),
    )
    .await
    .expect(
        "conductor should have executed the soft block and updated the soft commitment state \
         within 1000ms",
    );
}

#[tokio::test]
async fn fetch_from_later_celestia_height() {
    let test_conductor = spawn_conductor(CommitLevel::FirmOnly).await;

    mount_get_genesis_info!(
        test_conductor,
        sequencer_genesis_block_height: 1,
        celestia_base_block_height: 4,
        celestia_block_variance: 10,
    );

    mount_get_commitment_state!(
        test_conductor,
        firm: (
            number: 1,
            hash: [1; 64],
            parent: [0; 64],
        ),
        soft: (
            number: 1,
            hash: [1; 64],
            parent: [0; 64],
        ),
    );

    mount_sequencer_genesis!(test_conductor);

    mount_celestia_header_network_head!(
        test_conductor,
        height: 5u32,
    );

    mount_celestia_blobs!(
        test_conductor,
        celestia_height: 4,
        sequencer_height: 3,
    );

    mount_sequencer_commit!(
        test_conductor,
        height: 3u32,
    );

    mount_sequencer_validator_set!(test_conductor, height: 2u32);

    let execute_block = mount_executed_block!(
        test_conductor,
        number: 2,
        hash: [2; 64],
        parent: [1; 64],
    );

    let update_commitment_state = mount_update_commitment_state!(
        test_conductor,
        firm: (
            number: 2,
            hash: [2; 64],
            parent: [1; 64],
        ),
        soft: (
            number: 2,
            hash: [2; 64],
            parent: [1; 64],
        ),
    );

    timeout(
        Duration::from_millis(2000),
        join(
            execute_block.wait_until_satisfied(),
            update_commitment_state.wait_until_satisfied(),
        ),
    )
    .await
    .expect(
        "conductor should have executed the firm block and updated the firm commitment state \
         within 1000ms",
    );
}
