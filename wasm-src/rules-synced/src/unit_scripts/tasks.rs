use alloc::rc::Rc;
use core::cell::RefCell;

use spring::cus::{Axis, Duration, UnitCtx, WreckLevel};

use crate::random::unit_float;

use super::actions::face_from_position;
use super::common::{
    SIG_ATTACK, SIG_BUFF, SIG_DEATH, SIG_MOVE, SIG_STUN, move_piece, play_unit_sound, spin,
    stop_spin, turn,
};
use super::kinds::ScriptKind;
use super::{Pieces, TtldState};

pub(super) async fn bob_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_MOVE);
    let initial_delay = {
        let mut state = state.borrow_mut();
        if state.is_ork && state.first_bob_run {
            let delay = (ctx
                .with_spring(|spring| Ok(unit_float(spring)))
                .expect("unit-script random scope")
                * 464.0)
                .ceil() as u64;
            state.first_bob_run = false;
            Some(delay)
        } else {
            None
        }
    };
    if let Some(delay) = initial_delay {
        ctx.sleep(Duration::from_millis(delay)).await;
    }
    loop {
        let (piece, dead) = {
            let state = state.borrow();
            (state.pieces.torso, state.dead)
        };
        if dead {
            return;
        }
        for (offset, speed, millis) in BOB_STEPS {
            let (scale, time_scale) = {
                let state = state.borrow();
                (state.bob_scale, state.time_scale)
            };
            move_piece(&ctx, piece, Axis::Z, offset * scale, speed * scale);
            ctx.sleep(Duration::from_millis((millis * time_scale) as u64))
                .await;
        }
    }
}

const BOB_STEPS: [(f32, f32, f32); 6] = [
    (3.0, 20.0, 100.0),
    (4.0, 10.0, 66.0),
    (3.0, 10.0, 66.0),
    (0.0, 20.0, 100.0),
    (-1.0, 10.0, 66.0),
    (0.0, 10.0, 66.0),
];

pub(super) async fn attack_bob_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_ATTACK);
    let (piece, size, dead) = {
        let state = state.borrow();
        (state.pieces.torso, state.size_scale, state.dead)
    };
    if dead {
        return;
    }
    move_piece(
        &ctx,
        piece,
        Axis::Y,
        -10.0 - 10.0 * size,
        250.0 + 250.0 * size,
    );
    turn(
        &ctx,
        piece,
        Axis::Y,
        -20.0_f32.to_radians(),
        200.0_f32.to_radians(),
    );
    ctx.sleep(Duration::from_millis(100)).await;
    move_piece(&ctx, piece, Axis::Y, 0.0, 100.0 + 150.0 * size);
    turn(&ctx, piece, Axis::Y, 0.0, 120.0_f32.to_radians());
}

pub(super) async fn melee_attack_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    attack_bob_task(Rc::clone(&state), ctx.clone()).await;
    let is_sword = state.borrow().kind == Some(ScriptKind::Swordsman);
    if is_sword {
        attack_task(state, ctx).await;
    }
}

pub(super) async fn attack_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_ATTACK);
    let (pieces, kind, dead) = {
        let state = state.borrow();
        (state.pieces, state.kind, state.dead)
    };
    if dead {
        return;
    }
    move_piece(&ctx, pieces.hand_right, Axis::Y, -30.0, 1000.0);
    turn(
        &ctx,
        pieces.hand_right,
        Axis::X,
        -20.0_f32.to_radians(),
        200.0_f32.to_radians(),
    );
    turn(
        &ctx,
        pieces.hand_right,
        Axis::Z,
        30.0_f32.to_radians(),
        100.0_f32.to_radians(),
    );
    if kind == Some(ScriptKind::BloodMage) {
        turn(
            &ctx,
            pieces.torso,
            Axis::X,
            -(5.0
                + ctx
                    .with_spring(|spring| Ok(unit_float(spring)))
                    .expect("unit-script random scope")
                    * 2.0)
                .to_radians(),
            120.0_f32.to_radians(),
        );
        move_piece(
            &ctx,
            pieces.torso,
            Axis::Y,
            -8.0 - ctx
                .with_spring(|spring| Ok(unit_float(spring)))
                .expect("unit-script random scope")
                * 4.0,
            45.0 + ctx
                .with_spring(|spring| Ok(unit_float(spring)))
                .expect("unit-script random scope")
                * 10.0,
        );
    }
    ctx.sleep(Duration::from_millis(100)).await;
    move_piece(&ctx, pieces.hand_right, Axis::Y, 0.0, 500.0);
    turn(&ctx, pieces.hand_right, Axis::X, 0.0, 40.0_f32.to_radians());
    turn(&ctx, pieces.hand_right, Axis::Z, 0.0, 50.0_f32.to_radians());
    if kind == Some(ScriptKind::BloodMage) {
        turn(&ctx, pieces.torso, Axis::X, 0.0, 100.0_f32.to_radians());
        move_piece(&ctx, pieces.torso, Axis::Y, 0.0, 30.0);
    }
}

pub(super) async fn cast_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_ATTACK);
    let (pieces, level, tx, tz, unit, dead) = {
        let state = state.borrow();
        (
            state.pieces,
            state.cast_level,
            state.cast_tx,
            state.cast_tz,
            ctx.unit(),
            state.dead,
        )
    };
    if dead {
        return;
    }
    ctx.signal(SIG_MOVE);
    move_piece(&ctx, pieces.torso, Axis::Y, 0.0, 10.0);
    ctx.with_spring(|spring| {
        crate::engine::check(
            spring,
            spring.unit_control().clear_unit_goal(unit.0, true),
            "clear_unit_goal",
            unit.0,
        );
        Ok(())
    })
    .expect("unit-script goal scope");
    if level > 1 {
        ctx.with_spring(|spring| {
            face_from_position(spring, unit, tx, tz);
            Ok(())
        })
        .expect("unit-script facing scope");
    }
    raise_arms(&ctx, pieces, level);
    if level == 1 {
        ctx.sleep(Duration::from_millis(300)).await;
        ctx.sleep(Duration::from_millis(100)).await;
    } else if level == 2 {
        move_piece(&ctx, pieces.torso, Axis::Z, 25.0, 100.0);
        ctx.sleep(Duration::from_millis(300)).await;
        move_piece(&ctx, pieces.torso, Axis::Z, 30.0, 30.0);
        ctx.sleep(Duration::from_millis(60)).await;
        move_piece(&ctx, pieces.torso, Axis::Z, 0.0, 20.0);
        ctx.sleep(Duration::from_millis(60)).await;
        move_piece(&ctx, pieces.torso, Axis::Z, 0.0, 100.0);
    } else {
        move_piece(&ctx, pieces.torso, Axis::Z, 40.0, 120.0);
        ctx.sleep(Duration::from_millis(450)).await;
        ctx.sleep(Duration::from_millis(100)).await;
        move_piece(&ctx, pieces.torso, Axis::Z, 0.0, 80.0);
    }
    lower_arms(&ctx, pieces);
}

fn raise_arms(ctx: &UnitCtx, pieces: Pieces, level: i32) {
    move_piece(
        ctx,
        pieces.hand_right,
        Axis::X,
        -50.0,
        if level == 3 { 130.0 } else { 200.0 },
    );
    move_piece(
        ctx,
        pieces.hand_right,
        Axis::Z,
        40.0,
        if level == 3 { 110.0 } else { 150.0 },
    );
    move_piece(
        ctx,
        pieces.hand_left,
        Axis::X,
        50.0,
        if level == 3 { 130.0 } else { 200.0 },
    );
    move_piece(
        ctx,
        pieces.hand_left,
        Axis::Z,
        40.0,
        if level == 3 { 110.0 } else { 150.0 },
    );
    turn(
        ctx,
        pieces.hand_right,
        Axis::Z,
        25.0_f32.to_radians(),
        if level == 3 {
            120.0_f32.to_radians()
        } else {
            160.0_f32.to_radians()
        },
    );
    turn(
        ctx,
        pieces.hand_right,
        Axis::X,
        -82.0_f32.to_radians(),
        if level == 3 {
            200.0_f32.to_radians()
        } else {
            260.0_f32.to_radians()
        },
    );
    turn(
        ctx,
        pieces.hand_left,
        Axis::Z,
        50.0_f32.to_radians(),
        if level == 3 {
            120.0_f32.to_radians()
        } else {
            160.0_f32.to_radians()
        },
    );
}

fn lower_arms(ctx: &UnitCtx, pieces: Pieces) {
    move_piece(ctx, pieces.hand_right, Axis::X, 0.0, 100.0);
    move_piece(ctx, pieces.hand_right, Axis::Z, 0.0, 80.0);
    move_piece(ctx, pieces.hand_left, Axis::X, 0.0, 100.0);
    move_piece(ctx, pieces.hand_left, Axis::Z, 0.0, 80.0);
    turn(ctx, pieces.hand_right, Axis::Z, 0.0, 120.0_f32.to_radians());
    turn(ctx, pieces.hand_right, Axis::X, 0.0, 200.0_f32.to_radians());
    turn(ctx, pieces.hand_left, Axis::Z, 0.0, 120.0_f32.to_radians());
}

pub(super) async fn buff_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_BUFF);
    let duration = state.borrow().effect_frames.max(0) as u64;
    {
        let mut state = state.borrow_mut();
        state.bob_scale = state.static_bob_scale * 2.0;
        state.time_scale /= 2.0;
    }
    ctx.sleep(Duration::from_millis(duration.saturating_mul(1000) / 30))
        .await;
    let mut state = state.borrow_mut();
    state.bob_scale = state.static_bob_scale;
    state.time_scale = 1.0;
}

pub(super) async fn stun_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_STUN);
    let (piece, duration, direction, spin_speed) = {
        let mut state = state.borrow_mut();
        let direction = if ctx
            .with_spring(|spring| Ok(unit_float(spring)))
            .expect("unit-script random scope")
            < 0.5
        {
            -1.0
        } else {
            1.0
        };
        let spin_speed = 250.0
            + ctx
                .with_spring(|spring| Ok(unit_float(spring)))
                .expect("unit-script random scope")
                * 100.0;
        state.bob_scale = state.static_bob_scale * 0.1;
        (
            state.pieces.torso,
            state.effect_frames.max(0) as u64,
            direction,
            spin_speed,
        )
    };
    spin(
        &ctx,
        piece,
        Axis::Z,
        direction * spin_speed.to_radians(),
        400.0_f32.to_radians(),
    );
    turn(
        &ctx,
        piece,
        Axis::Y,
        90.0_f32.to_radians(),
        90.0_f32.to_radians(),
    );
    ctx.sleep(Duration::from_millis(duration.saturating_mul(1000) / 30))
        .await;
    stop_spin(&ctx, piece, Axis::Z, 400.0_f32.to_radians());
    turn(&ctx, piece, Axis::Y, 0.0, 180.0_f32.to_radians());
    let static_bob_scale = state.borrow().static_bob_scale;
    state.borrow_mut().bob_scale = static_bob_scale;
    ctx.sleep(Duration::from_millis(1000)).await;
    turn(&ctx, piece, Axis::Z, 0.0, 400.0_f32.to_radians());
}

pub(super) async fn death_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_DEATH);
    let (piece, scale, size) = {
        let state = state.borrow();
        (state.pieces.torso, state.bob_scale, state.size_scale)
    };
    turn(
        &ctx,
        piece,
        Axis::Y,
        90.0_f32.to_radians(),
        250.0_f32.to_radians(),
    );
    move_piece(&ctx, piece, Axis::Z, 2.0 * scale, 20.0 * scale);
    ctx.sleep(Duration::from_millis(200)).await;
    turn(
        &ctx,
        piece,
        Axis::Z,
        90.0_f32.to_radians(),
        180.0_f32.to_radians(),
    );
    move_piece(&ctx, piece, Axis::Z, -50.0 * size, 30.0 * size);
    ctx.sleep(Duration::from_millis(1500)).await;
    state.borrow_mut().death_pending = false;
    crate::with_global(|global| crate::combat::forget(global, ctx.unit().0));
    ctx.set_killed_finished(WreckLevel::NoWreck);
}

pub(super) async fn house_death_task(state: Rc<RefCell<TtldState>>, ctx: UnitCtx) {
    ctx.set_signal_mask(SIG_DEATH);
    let piece = state.borrow().pieces.house;
    ctx.with_spring(|spring| {
        play_unit_sound(spring, ctx.unit(), "sounds/house_fall.wav", 10.0);
        Ok(())
    })
    .expect("unit-script sound scope");
    move_piece(&ctx, piece, Axis::Z, -250.0, 250.0);
    turn(
        &ctx,
        piece,
        Axis::X,
        (ctx.with_spring(|spring| Ok(unit_float(spring)))
            .expect("unit-script random scope")
            - 0.5)
            / 2.0,
        ctx.with_spring(|spring| Ok(unit_float(spring)))
            .expect("unit-script random scope")
            / 2.0,
    );
    turn(
        &ctx,
        piece,
        Axis::Y,
        (ctx.with_spring(|spring| Ok(unit_float(spring)))
            .expect("unit-script random scope")
            - 0.5)
            / 2.0,
        ctx.with_spring(|spring| Ok(unit_float(spring)))
            .expect("unit-script random scope")
            / 2.0,
    );
    turn(
        &ctx,
        piece,
        Axis::Z,
        (ctx.with_spring(|spring| Ok(unit_float(spring)))
            .expect("unit-script random scope")
            - 0.5)
            / 4.0,
        ctx.with_spring(|spring| Ok(unit_float(spring)))
            .expect("unit-script random scope")
            / 4.0,
    );
    ctx.sleep(Duration::from_millis(1200)).await;
    state.borrow_mut().death_pending = false;
    ctx.set_killed_finished(WreckLevel::NoWreck);
}
