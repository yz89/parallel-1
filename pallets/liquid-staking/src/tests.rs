use crate::{
    mock::*,
    types::{MatchingLedger, StakingLedger, UnlockChunk},
    *,
};

use frame_support::{assert_noop, assert_ok, storage::with_transaction, traits::Hooks};

use primitives::{
    tokens::{KSM, XKSM},
    ump::RewardDestination,
    Balance, Rate, Ratio,
};
use sp_runtime::{
    traits::{One, Zero},
    MultiAddress::Id,
    TransactionOutcome,
};
use xcm_simulator::TestExt;

#[test]
fn stake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(10f64)));
        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: ksm(9.95f64),
                total_unstake_amount: 0,
            }
        );

        // Check balance is correct
        assert_eq!(<Test as Config>::Assets::balance(KSM, &ALICE), ksm(90f64));
        assert_eq!(
            <Test as Config>::Assets::balance(XKSM, &ALICE),
            ksm(109.95f64)
        );

        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &LiquidStaking::account_id()),
            ksm(10f64)
        );

        with_transaction(|| {
            LiquidStaking::do_advance_era(1).unwrap();
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                0,
                Response::ExecutionResult(None),
            )
            .unwrap();
            TransactionOutcome::Commit(0)
        });

        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &LiquidStaking::account_id()),
            ksm(0.05f64)
        );

        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: 0,
                total_unstake_amount: 0,
            }
        );
        let derivative_index = <Test as Config>::DerivativeIndex::get();
        assert_eq!(
            StakingLedgers::<Test>::get(&0).unwrap(),
            StakingLedger {
                stash: LiquidStaking::derivative_sovereign_account_id(derivative_index),
                total: ksm(9.95f64),
                active: ksm(9.95f64),
                unlocking: vec![],
                claimed_rewards: vec![]
            }
        );

        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(10f64)));

        with_transaction(|| {
            LiquidStaking::do_advance_era(1).unwrap();
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                1,
                Response::ExecutionResult(None),
            )
            .unwrap();
            TransactionOutcome::Commit(0)
        });

        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &LiquidStaking::account_id()),
            ksm(0.1f64)
        );

        assert_eq!(
            StakingLedgers::<Test>::get(&0).unwrap(),
            StakingLedger {
                stash: LiquidStaking::derivative_sovereign_account_id(derivative_index),
                total: ksm(19.9f64),
                active: ksm(19.9f64),
                unlocking: vec![],
                claimed_rewards: vec![]
            }
        );
    })
}

#[test]
fn unstake_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(10f64)));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(6f64)));

        // Check storage is correct
        assert_eq!(ExchangeRate::<Test>::get(), Rate::one());
        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: ksm(9.95f64),
                total_unstake_amount: ksm(6f64),
            }
        );

        assert_eq!(
            Unlockings::<Test>::get(ALICE).unwrap(),
            vec![UnlockChunk {
                value: ksm(6f64),
                era: 4
            }]
        );

        with_transaction(|| {
            LiquidStaking::do_advance_era(1).unwrap();
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                0,
                Response::ExecutionResult(None),
            )
            .unwrap();
            TransactionOutcome::Commit(0)
        });

        assert_eq!(
            MatchingPool::<Test>::get(),
            MatchingLedger {
                total_stake_amount: 0,
                total_unstake_amount: 0,
            }
        );

        let derivative_index = <Test as Config>::DerivativeIndex::get();
        assert_eq!(
            StakingLedgers::<Test>::get(&0).unwrap(),
            StakingLedger {
                stash: LiquidStaking::derivative_sovereign_account_id(derivative_index),
                total: ksm(3.95f64),
                active: ksm(3.95f64),
                unlocking: vec![],
                claimed_rewards: vec![]
            }
        );
        // Just make it 1 to calculate.
        ExchangeRate::<Test>::set(Rate::one());
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(3.95f64)));

        assert_eq!(
            Unlockings::<Test>::get(ALICE).unwrap(),
            vec![
                UnlockChunk {
                    value: ksm(6f64),
                    era: 4
                },
                UnlockChunk {
                    value: ksm(3.95f64),
                    era: 5
                }
            ]
        );

        with_transaction(|| {
            LiquidStaking::do_advance_era(1).unwrap();
            LiquidStaking::notification_received(
                pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                1,
                Response::ExecutionResult(None),
            )
            .unwrap();
            TransactionOutcome::Commit(0)
        });

        assert_eq!(
            StakingLedgers::<Test>::get(&0).unwrap(),
            StakingLedger {
                stash: LiquidStaking::derivative_sovereign_account_id(derivative_index),
                total: ksm(3.95),
                active: 0,
                unlocking: vec![UnlockChunk {
                    value: ksm(3.95),
                    era: 5
                }],
                claimed_rewards: vec![]
            }
        );
    })
}

enum StakeOp {
    Stake(Balance),
    Unstake(Balance),
}

impl StakeOp {
    fn execute(self) {
        match self {
            Self::Stake(amount) => LiquidStaking::stake(Origin::signed(ALICE), amount).unwrap(),
            Self::Unstake(amount) => LiquidStaking::unstake(Origin::signed(ALICE), amount).unwrap(),
        };
    }
}

#[test]
fn test_matching_should_work() {
    use StakeOp::*;
    TestNet::reset();
    ParaA::execute_with(|| {
        let test_case: Vec<(Vec<StakeOp>, Balance, Balance, (Balance, Balance, Balance))> = vec![
            (
                vec![Stake(ksm(5000f64)), Unstake(ksm(1000f64))],
                0,
                0,
                (ksm(3975f64), 0, 0),
            ),
            // Calculate right here.
            (
                vec![Unstake(ksm(10f64)), Unstake(ksm(5f64)), Stake(ksm(10f64))],
                ksm(3975f64),
                0,
                (0, 0, ksm(5.05f64)),
            ),
            // (vec![], 0, (0, 0, 0)),
        ];
        for (i, (stake_ops, _bonding_amount, unbonding_amount, matching_result)) in
            test_case.into_iter().enumerate()
        {
            stake_ops.into_iter().for_each(StakeOp::execute);
            assert_eq!(
                LiquidStaking::matching_pool().matching(unbonding_amount),
                Ok(matching_result)
            );
            with_transaction(|| {
                LiquidStaking::do_advance_era(1).unwrap();
                LiquidStaking::notification_received(
                    pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
                    i.try_into().unwrap(),
                    Response::ExecutionResult(None),
                )
                .unwrap();
                TransactionOutcome::Commit(0)
            });
        }
    });
}

#[test]
fn test_transact_bond_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(2000f64),));
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(3f64),
            RewardDestination::Staked
        ));

        ParaSystem::assert_has_event(mock::Event::LiquidStaking(crate::Event::Bonding(
            derivative_index,
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(3f64),
            RewardDestination::Staked,
        )));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(3f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(3f64));
    });
}

#[test]
fn test_transact_bond_extra_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(4000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(2f64),
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));

        assert_ok!(LiquidStaking::bond_extra(
            Origin::signed(ALICE),
            derivative_index,
            ksm(3f64)
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(5f64));
    });
}

#[test]
fn test_transact_unbond_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(5f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(2f64)
        ));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(2f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(5f64));
        assert_eq!(ledger.active, ksm(3f64));
    });
}

#[test]
fn test_transact_withdraw_unbonded_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(2000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(5f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(2f64)
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            1,
            Response::ExecutionResult(None),
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(5f64));
        assert_eq!(ledger.active, ksm(3f64));
        assert_eq!(ledger.unlocking.len(), 1);

        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(2f64),
        )));

        pallet_staking::CurrentEra::<KusamaRuntime>::put(
            <KusamaRuntime as pallet_staking::Config>::BondingDuration::get(),
        );
    });

    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::force_set_current_era(
            Origin::root(),
            <KusamaRuntime as pallet_staking::Config>::BondingDuration::get(),
        ));

        assert_ok!(LiquidStaking::withdraw_unbonded(
            Origin::signed(BOB),
            derivative_index,
            0
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(3f64));
        assert_eq!(ledger.active, ksm(3f64));
        assert_eq!(ledger.unlocking.len(), 0);
    });
}

#[test]
fn test_transact_rebond_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(6000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(10f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        assert_ok!(LiquidStaking::unbond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(5f64)
        ));
        assert_ok!(LiquidStaking::rebond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(3f64)
        ));
    });

    Relay::execute_with(|| {
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(10f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Unbonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(5f64),
        )));
        RelaySystem::assert_has_event(RelayEvent::Staking(RelayStakingEvent::Bonded(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            ksm(3f64),
        )));
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(10f64));
        assert_eq!(ledger.active, ksm(8f64));
    });
}

#[test]
fn test_transact_nominate_work() {
    TestNet::reset();
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(4000f64),));

        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(10f64),
            RewardDestination::Staked
        ));

        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));

        assert_ok!(LiquidStaking::nominate(
            Origin::signed(ALICE),
            derivative_index,
            vec![ALICE, BOB],
        ));
    });

    Relay::execute_with(|| {
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, ksm(10f64));
        let nominators = RelayStaking::nominators(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(nominators.targets, vec![ALICE, BOB]);
    });
}

#[test]
fn test_transfer_bond() {
    TestNet::reset();
    let xcm_transfer_amount = ksm(10f64);
    let derivative_index = <Test as Config>::DerivativeIndex::get();
    ParaA::execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(2000f64),));
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            xcm_transfer_amount,
            RewardDestination::Staked
        ));
        // print_events::<Test>("ParaA");
    });
    Relay::execute_with(|| {
        // print_events::<kusama_runtime::Runtime>("Relay");
        let ledger = RelayStaking::ledger(LiquidStaking::derivative_sovereign_account_id(
            derivative_index,
        ))
        .unwrap();
        assert_eq!(ledger.total, xcm_transfer_amount);
        assert_eq!(
            RelayBalances::free_balance(LiquidStaking::derivative_sovereign_account_id(
                derivative_index
            )),
            xcm_transfer_amount
        );
        assert_eq!(
            RelayBalances::usable_balance(LiquidStaking::derivative_sovereign_account_id(
                derivative_index
            )),
            0
        );
    });
}

#[test]
fn update_market_cap_should_not_work_if_with_invalid_param() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LiquidStaking::update_market_cap(Origin::root(), Zero::zero()),
            Error::<Test>::InvalidCap
        );
    })
}

#[test]
fn update_reserve_factor_should_not_work_if_with_invalid_param() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            LiquidStaking::update_reserve_factor(Origin::root(), Ratio::zero()),
            Error::<Test>::InvalidFactor
        );
        assert_noop!(
            LiquidStaking::update_reserve_factor(Origin::root(), Ratio::one()),
            Error::<Test>::InvalidFactor
        );
    })
}

#[test]
fn claim_for_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), ksm(10f64)));
        assert_eq!(<Test as Config>::Assets::balance(KSM, &ALICE), ksm(90f64));

        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(1f64)));
        assert_ok!(LiquidStaking::unstake(Origin::signed(ALICE), ksm(3.95f64)));
        assert_eq!(
            Unlockings::<Test>::get(ALICE).unwrap(),
            vec![UnlockChunk {
                value: ksm(4.95f64),
                era: 4
            },]
        );

        assert_noop!(
            LiquidStaking::claim_for(Origin::signed(BOB), Id(ALICE)),
            Error::<Test>::NothingToClaim
        );

        let derivative_index = <Test as Config>::DerivativeIndex::get();
        assert_ok!(LiquidStaking::bond(
            Origin::signed(ALICE),
            derivative_index,
            ksm(3f64),
            RewardDestination::Staked
        ));
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));
        MaxWithdrewUnbondedEra::<Test>::put(4);
        assert_ok!(LiquidStaking::withdraw_unbonded(
            Origin::signed(BOB),
            derivative_index,
            0
        ));

        assert_ok!(LiquidStaking::claim_for(Origin::signed(BOB), Id(ALICE)));
        assert_eq!(
            <Test as Config>::Assets::balance(KSM, &ALICE),
            ksm(90f64) + ksm(4.95f64)
        );

        assert!(Unlockings::<Test>::get(ALICE).is_none());
    })
}

#[test]
fn test_on_initialize_work() {
    new_test_ext().execute_with(|| {
        let derivative_index = <Test as Config>::DerivativeIndex::get();
        let xcm_fees = XcmFees::get();
        let reserve_factor = LiquidStaking::reserve_factor();

        // 1.1 stake
        let bond_amount = ksm(10f64);
        assert_ok!(LiquidStaking::stake(Origin::signed(ALICE), bond_amount));
        let total_stake_amount = bond_amount - xcm_fees - reserve_factor.mul_floor(bond_amount);

        // 1.2 on_initialize_bond
        let total_era_blocknumbers = <Test as Config>::EraLength::get();
        assert_eq!(total_era_blocknumbers, 10);
        RelayChainBlockNumberProvider::set(total_era_blocknumbers);
        LiquidStaking::on_initialize(System::block_number());
        assert_eq!(EraStartBlock::<Test>::get(), total_era_blocknumbers);
        assert_eq!(CurrentEra::<Test>::get(), 1);
        assert_eq!(LiquidStaking::staking_ledgers(derivative_index), None);
        assert_eq!(
            LiquidStaking::matching_pool(),
            MatchingLedger {
                total_stake_amount,
                total_unstake_amount: 0,
            }
        );

        // 1.3 notification_received bond
        assert_ok!(LiquidStaking::notification_received(
            pallet_xcm::Origin::Response(MultiLocation::parent()).into(),
            0,
            Response::ExecutionResult(None),
        ));

        let staking_ledger = <StakingLedger<AccountId, BalanceOf<Test>>>::new(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            total_stake_amount,
        );
        assert_eq!(
            LiquidStaking::staking_ledgers(derivative_index).unwrap(),
            staking_ledger
        );

        assert_eq!(LiquidStaking::matching_pool(), MatchingLedger::default());
    })
}

#[test]
fn test_force_set_staking_ledger_work() {
    new_test_ext().execute_with(|| {
        let derivative_index = <Test as Config>::DerivativeIndex::get();
        let bond_amount = 100;
        let bond_extra_amount = 50;
        let mut staking_ledger = <StakingLedger<AccountId, BalanceOf<Test>>>::new(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            bond_amount,
        );
        assert_noop!(
            LiquidStaking::force_set_staking_ledger(
                Origin::signed(ALICE),
                derivative_index,
                staking_ledger.clone()
            ),
            Error::<Test>::NotBonded
        );
        StakingLedgers::<Test>::insert(derivative_index, staking_ledger.clone());
        assert_eq!(
            LiquidStaking::staking_ledgers(derivative_index).unwrap(),
            staking_ledger.clone()
        );
        staking_ledger.bond_extra(bond_extra_amount);
        assert_ok!(LiquidStaking::force_set_staking_ledger(
            Origin::signed(ALICE),
            derivative_index,
            staking_ledger.clone()
        ));

        assert_noop!(
            LiquidStaking::force_set_staking_ledger(
                Origin::signed(ALICE),
                derivative_index,
                staking_ledger.clone()
            ),
            Error::<Test>::StakingLedgerLocked
        );

        LiquidStaking::on_finalize(1);

        assert_ok!(LiquidStaking::force_set_staking_ledger(
            Origin::signed(ALICE),
            derivative_index,
            staking_ledger.clone()
        ));

        let new_staking_ledger = <StakingLedger<AccountId, BalanceOf<Test>>>::new(
            LiquidStaking::derivative_sovereign_account_id(derivative_index),
            bond_amount + bond_extra_amount,
        );
        assert_eq!(
            LiquidStaking::staking_ledgers(derivative_index).unwrap(),
            new_staking_ledger
        );
    })
}

#[test]
fn test_force_set_era_start_block_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(EraStartBlock::<Test>::get(), 0);
        assert_ok!(LiquidStaking::force_set_era_start_block(Origin::root(), 11));
        assert_eq!(EraStartBlock::<Test>::get(), 11);
    })
}

#[test]
fn test_force_set_current_era_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(CurrentEra::<Test>::get(), 0);
        assert_ok!(LiquidStaking::force_set_current_era(Origin::root(), 12));
        assert_eq!(CurrentEra::<Test>::get(), 12);
    })
}
