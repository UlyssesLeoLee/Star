//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/Physis (MIT OR Apache-2.0) — GVPE 物理引擎内核
//! per 2026-11-14: 全 3D 主题 (Physis WASM + three.js 背景层)
//! Source: E:\Physis @ dev 3e2cf93 (gvpe-math/core/memory/dynamics 4 crate)
//! ═══════════════════════════════════════════════════════════════════

//! `gvpe-math` 单元测试 + 布局验证。

use super::*;

// ============================================================================
// 属性测试 (proptest) — 数学/几何不变量
//
// 覆盖四类：
//   1. 四元数 (Quat): 单位 / 乘法结合性 / 单位元 / 旋转等效矩阵
//   2. 旋转矩阵 (Mat3): 正交性 / 保长性 / 行列式 = 1
//   3. 投影 (Vec3::project_onto): 幂等性 / 残差正交
//   4. 三角恒等式: sin²x + cos²x = 1（通过 axis-angle 单位四元数隐式验证）
//
// 设计原则：
//   - 仅追加文件，不改 src / Cargo.toml (proptest 已在 dev-deps)。
//   - 用 prop_assert! 避免 panic；退化输入用 prop_assume! 跳过。
//   - 输入范围限制到物理合理域 (-1e3..1e3) / (-π..π)，避免 NaN/Inf 雪崩。
//   - 浮点容差使用 prop_assert! 显式比对，不依赖 assert_eq!。
// ============================================================================

use proptest::prelude::*;

// ============================================================================
// Vec3 测试
// ============================================================================

#[test]
fn vec3_size_and_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Vec3>(), 12);
    assert_eq!(align_of::<Vec3>(), 4);
}

#[test]
fn vec3_arithmetic() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
    assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
    assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
    assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
}

#[test]
fn vec3_dot_and_cross() {
    let a = Vec3::new(1.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 1.0, 0.0);

    assert!(a.dot(b).abs() < 1e-6);
    assert!((a.dot(a) - 1.0).abs() < 1e-6);

    let c = a.cross(b);
    assert_eq!(c, Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn vec3_normalize() {
    let v = Vec3::new(3.0, 0.0, 4.0);
    let n = v.normalize();
    assert!((n.length() - 1.0).abs() < 1e-6);

    // 零向量归一化返回零
    let zero = Vec3::ZERO.normalize();
    assert_eq!(zero, Vec3::ZERO);
}

// ============================================================================
// Quat 测试
// ============================================================================

#[test]
fn quat_size_and_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Quat>(), 16);
    assert_eq!(align_of::<Quat>(), 16, "Quat 应 16 字节对齐以支持 SIMD");
}

#[test]
fn quat_identity_is_unit() {
    let q = Quat::IDENTITY;
    assert!((q.length_squared() - 1.0).abs() < 1e-6);
}

#[test]
fn quat_conjugate() {
    let q = Quat::new(0.1, 0.2, 0.3, 0.4).normalize();
    let c = q.conjugate();
    assert!((c.x - (-q.x)).abs() < 1e-6);
    assert!((c.y - (-q.y)).abs() < 1e-6);
    assert!((c.z - (-q.z)).abs() < 1e-6);
    assert!((c.w - q.w).abs() < 1e-6);
}

#[test]
fn quat_rotate_180_x_axis() {
    // 围绕 X 轴 180 度
    let axis = Vec3::X;
    let q = Quat::from_axis_angle(axis, std::f32::consts::PI);
    let v = Vec3::new(0.0, 1.0, 0.0);
    let r = q.rotate_vec3(v);
    assert!((r.x - 0.0).abs() < 1e-5);
    assert!((r.y - (-1.0)).abs() < 1e-5);
    assert!((r.z - 0.0).abs() < 1e-5);
}

// ============================================================================
// Mat3 测试
// ============================================================================

#[test]
fn mat3_size_and_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Mat3>(), 36);
    assert_eq!(align_of::<Mat3>(), 4);
}

#[test]
fn mat3_identity_mul() {
    let m = Mat3::IDENTITY;
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(m.mul_vec3(v), v);
}

#[test]
fn mat3_inverse_works() {
    // 简单测试矩阵
    let m = Mat3::new(2.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 4.0);
    let inv = m.inverse().unwrap();
    // 逆矩阵 × 原矩阵 = 单位
    let prod = inv.mul_mat3(m);
    assert!((prod.m[0].x - 1.0).abs() < 1e-5);
    assert!((prod.m[1].y - 1.0).abs() < 1e-5);
    assert!((prod.m[2].z - 1.0).abs() < 1e-5);
    assert!(prod.m[0].y.abs() < 1e-5);
    assert!(prod.m[1].x.abs() < 1e-5);
}

#[test]
fn mat3_singular_returns_none() {
    // 奇异矩阵（所有元素相同）
    let m = Mat3::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    assert!(m.inverse().is_none());
}

// ============================================================================
// Transform 测试
// ============================================================================

#[test]
fn transform_size_and_alignment() {
    use std::mem::{align_of, size_of};
    // 布局：Vec3(12) + padding(4, 凑齐 16 字节对齐) + Quat(16) = 32 字节。
    // `rotation` 需要 16 字节对齐以支持 SIMD 优化，故必须插入 4 字节 padding。
    assert_eq!(size_of::<Transform>(), 32);
    assert_eq!(
        align_of::<Transform>(),
        16,
        "Transform 应 16 字节对齐以支持 SIMD"
    );
}

#[test]
fn transform_identity() {
    let t = Transform::IDENTITY;
    let v = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(t.transform_vec3(v), v);
}

#[test]
fn transform_inverse() {
    let t = Transform::new(Vec3::new(10.0, 20.0, 30.0), Quat::IDENTITY);
    let inv = t.inverse();
    let v = Vec3::new(1.0, 2.0, 3.0);
    // 应用 t 然后 inv 应回到原值
    let result = inv.transform_vec3(t.transform_vec3(v));
    assert!((result.x - v.x).abs() < 1e-5);
    assert!((result.y - v.y).abs() < 1e-5);
    assert!((result.z - v.z).abs() < 1e-5);
}

// ============================================================================
// Aabb 测试
// ============================================================================

#[test]
fn aabb_size_and_alignment() {
    use std::mem::{align_of, size_of};
    assert_eq!(size_of::<Aabb>(), 24);
    assert_eq!(align_of::<Aabb>(), 4);
}

#[test]
fn aabb_overlaps() {
    let a = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let b = Aabb::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    let c = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));

    assert!(a.overlaps(b));
    assert!(b.overlaps(a));
    assert!(!a.overlaps(c));
    assert!(!c.overlaps(a));
}

#[test]
fn aabb_contains() {
    let a = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    assert!(a.contains(Vec3::new(0.5, 0.5, 0.5)));
    assert!(a.contains(Vec3::new(0.0, 0.0, 0.0))); // 边界
    assert!(!a.contains(Vec3::new(2.0, 0.5, 0.5)));
}

#[test]
fn aabb_expand_and_merge() {
    let mut a = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
    a = a.expand_to_include(Vec3::new(2.0, 0.5, 0.5));
    assert!(a.contains(Vec3::new(2.0, 0.5, 0.5)));

    let b = Aabb::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    let merged = a.merged(b);
    assert!(merged.contains(Vec3::new(2.0, 0.5, 0.5)));
    assert!(merged.contains(Vec3::new(1.5, 1.5, 1.5)));
}

// ============================================================================
// v0.7 新增 API 测试
// ============================================================================

// ---- Quat::from_euler_ypr ----

#[test]
fn quat_from_euler_ypr_zero() {
    let q = Quat::from_euler_ypr(0.0, 0.0, 0.0);
    assert!((q.x - 0.0).abs() < 1e-6);
    assert!((q.y - 0.0).abs() < 1e-6);
    assert!((q.z - 0.0).abs() < 1e-6);
    assert!((q.w - 1.0).abs() < 1e-6);
}

#[test]
fn quat_from_euler_ypr_pure_yaw() {
    // yaw = π/2 绕 Z 轴，pitch = roll = 0
    let q = Quat::from_euler_ypr(std::f32::consts::FRAC_PI_2, 0.0, 0.0);
    // X 轴应被旋转到 Y 轴
    let rotated = q.rotate_vec3(Vec3::X);
    assert!((rotated.x - 0.0).abs() < 1e-5);
    assert!((rotated.y - 1.0).abs() < 1e-5);
    assert!((rotated.z - 0.0).abs() < 1e-5);
}

#[test]
fn quat_from_euler_ypr_compose() {
    // 小角度避免 gimbal lock
    let ypr = [0.3_f32, 0.2, 0.1];
    let q = Quat::from_euler_ypr(ypr[0], ypr[1], ypr[2]);
    let back = q.to_euler_ypr();
    assert!((back[0] - ypr[0]).abs() < 1e-5);
    assert!((back[1] - ypr[1]).abs() < 1e-5);
    assert!((back[2] - ypr[2]).abs() < 1e-5);
}

// ---- Quat::to_euler_ypr ----

#[test]
fn quat_to_euler_ypr_identity() {
    let ypr = Quat::IDENTITY.to_euler_ypr();
    assert!(ypr[0].abs() < 1e-6);
    assert!(ypr[1].abs() < 1e-6);
    assert!(ypr[2].abs() < 1e-6);
}

#[test]
fn quat_to_euler_ypr_gimbal_lock() {
    // pitch = π/2：构造绕 Y 轴 90° 旋转（只设 pitch）
    let q = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
    let ypr = q.to_euler_ypr();
    assert!(!ypr[0].is_nan(), "yaw 不应为 NaN");
    assert!(!ypr[1].is_nan(), "pitch 不应为 NaN");
    assert!(!ypr[2].is_nan(), "roll 不应为 NaN");
    // pitch 应接近 +π/2（f32 精度下可能走 asin 分支，asin(0.99999994) ≈ 1.5704，
    // 差 ~3.4e-4 是 f32 ULP 误差的本质，不是实现 bug；放宽到 1e-3）
    assert!(
        (ypr[1] - std::f32::consts::FRAC_PI_2).abs() < 1e-3,
        "pitch 偏离 π/2 过大：ypr[1] = {}",
        ypr[1]
    );
    // pitch = -π/2 也应不爆 NaN
    let q_neg = Quat::from_axis_angle(Vec3::Y, -std::f32::consts::FRAC_PI_2);
    let ypr_neg = q_neg.to_euler_ypr();
    assert!(
        (ypr_neg[1] - (-std::f32::consts::FRAC_PI_2)).abs() < 1e-3,
        "pitch 偏离 -π/2 过大：ypr_neg[1] = {}",
        ypr_neg[1]
    );
}

// ---- Vec3::splat ----

#[test]
fn vec3_splat() {
    let v = Vec3::splat(2.5);
    assert_eq!(v, Vec3::new(2.5, 2.5, 2.5));
    let z = Vec3::splat(0.0);
    assert_eq!(z, Vec3::ZERO);
    let neg = Vec3::splat(-1.0);
    assert_eq!(neg, Vec3::new(-1.0, -1.0, -1.0));
}

#[test]
fn vec3_splat_arithmetic() {
    // 验证 splat 出来的向量与显式构造语义一致
    let s = Vec3::splat(3.0);
    assert_eq!(s + s, Vec3::new(6.0, 6.0, 6.0));
    assert_eq!(s * 2.0, Vec3::splat(6.0));
    assert!((s.length_squared() - 27.0).abs() < 1e-6); // 3*3 + 3*3 + 3*3 = 27
}

// ---- Mat3::from_diagonal ----

#[test]
fn mat3_from_diagonal() {
    let m = Mat3::from_diagonal(Vec3::new(2.0, 3.0, 4.0));
    // 对角矩阵 × 向量 = 逐分量缩放
    let r = m.mul_vec3(Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(r, Vec3::new(2.0, 3.0, 4.0));
    // 非对角元素为 0
    assert!(m.m[0].y.abs() < 1e-6);
    assert!(m.m[1].x.abs() < 1e-6);
    assert!(m.m[2].x.abs() < 1e-6);
    // 验证对角项位置
    assert!((m.m[0].x - 2.0).abs() < 1e-6);
    assert!((m.m[1].y - 3.0).abs() < 1e-6);
    assert!((m.m[2].z - 4.0).abs() < 1e-6);
}

// ---- Mat3::from_basis ----

#[test]
fn mat3_from_basis() {
    // 三轴列向量 = 坐标轴
    let m = Mat3::from_basis(Vec3::X, Vec3::Y, Vec3::Z);
    // 应为 IDENTITY（行优先存储：row0 = (1,0,0)）
    assert_eq!(m, Mat3::IDENTITY);
}

#[test]
fn mat3_from_basis_columns() {
    // 验证列向量语义：m * X = x, m * Y = y, m * Z = z
    let x = Vec3::new(1.0, 0.0, 0.0);
    let y = Vec3::new(0.0, 1.0, 0.0);
    let z = Vec3::new(0.0, 0.0, 1.0);
    let m = Mat3::from_basis(x, y, z);
    assert_eq!(m.mul_vec3(Vec3::X), x);
    assert_eq!(m.mul_vec3(Vec3::Y), y);
    assert_eq!(m.mul_vec3(Vec3::Z), z);
}

// ---- Quat::to_mat3 ----

#[test]
fn quat_to_mat3_matches_from_quat() {
    // 三个不同四元数：IDENTITY、纯 yaw、复合小角度
    let qs = [
        Quat::IDENTITY,
        Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_2),
        Quat::from_euler_ypr(0.3, 0.2, 0.1),
    ];
    for &q in &qs {
        let a = q.to_mat3();
        let b = Mat3::from_quat(q);
        assert_eq!(a, b, "to_mat3 必须 = from_quat");
        // 再验证语义：m * v == q.rotate_vec3(v)
        let v = Vec3::new(1.0, 2.0, 3.0);
        let m_v = a.mul_vec3(v);
        let q_v = q.rotate_vec3(v);
        assert!((m_v.x - q_v.x).abs() < 1e-5);
        assert!((m_v.y - q_v.y).abs() < 1e-5);
        assert!((m_v.z - q_v.z).abs() < 1e-5);
    }
}

// ---- Vec3::distance / distance_squared ----

#[test]
fn vec3_distance_zero() {
    let p = Vec3::new(1.0, 2.0, 3.0);
    assert!(p.distance(p).abs() < 1e-6);
    assert!(p.distance_squared(p).abs() < 1e-6);
}

#[test]
fn vec3_distance_axis_aligned() {
    // (1,2,3) → (4,6,3)：差 = (3, 4, 0)，距离 = 5
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 6.0, 3.0);
    assert!((a.distance(b) - 5.0).abs() < 1e-6);
    assert!((a.distance_squared(b) - 25.0).abs() < 1e-6);
}

#[test]
fn vec3_distance_squared_no_sqrt() {
    // 验证 distance_squared 等于差向量的 length_squared
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 6.0, 8.0);
    let ds = a.distance_squared(b);
    let expected = (b - a).length_squared();
    assert!((ds - expected).abs() < 1e-6);
    // 3² + 4² + 5² = 9 + 16 + 25 = 50
    assert!((ds - 50.0).abs() < 1e-6);
}

// ---- Aabb::from_points ----

#[test]
fn aabb_from_points_empty() {
    let pts: [Vec3; 0] = [];
    assert!(Aabb::from_points(&pts).is_none());
}

#[test]
fn aabb_from_points_single() {
    let pts = [Vec3::new(1.0, 2.0, 3.0)];
    let a = Aabb::from_points(&pts).unwrap();
    assert_eq!(a.min, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(a.max, Vec3::new(1.0, 2.0, 3.0));
    // 零体积 AABB 与 from_point 一致
    assert_eq!(a, Aabb::from_point(pts[0]));
}

#[test]
fn aabb_from_points_two_corners() {
    // 两个对角点
    let pts = [Vec3::new(-1.0, -2.0, -3.0), Vec3::new(4.0, 5.0, 6.0)];
    let a = Aabb::from_points(&pts).unwrap();
    assert_eq!(a.min, Vec3::new(-1.0, -2.0, -3.0));
    assert_eq!(a.max, Vec3::new(4.0, 5.0, 6.0));
}

#[test]
fn aabb_from_points_three_points() {
    // 三个点，验证 min/max 正确（顺序无关）
    let pts = [
        Vec3::new(1.0, 5.0, -2.0),
        Vec3::new(-3.0, 2.0, 4.0),
        Vec3::new(0.0, 0.0, 0.0),
    ];
    let a = Aabb::from_points(&pts).unwrap();
    assert_eq!(a.min, Vec3::new(-3.0, 0.0, -2.0));
    assert_eq!(a.max, Vec3::new(1.0, 5.0, 4.0));
    // 三个点都应被包含
    for &p in &pts {
        assert!(a.contains(p), "AABB 必须包含点 {p:?}");
    }
}

// ============================================================================
// Pod 派生验证（bytemuck）
// ============================================================================

#[test]
fn all_types_are_pod() {
    use bytemuck::Pod;
    fn assert_pod<T: Pod>() {}
    assert_pod::<Vec3>();
    assert_pod::<Quat>();
    assert_pod::<Mat3>();
    assert_pod::<Aabb>();
    assert_pod::<Transform>();
}

// ===== v0.8 API surface 扩充测试 =====

#[test]
fn vec3_min_component_wise() {
    let a = Vec3::new(1.0, 5.0, -3.0);
    let b = Vec3::new(2.0, -1.0, 4.0);
    let m = Vec3::min(a, b);
    assert_eq!(m, Vec3::new(1.0, -1.0, -3.0));
}

#[test]
fn vec3_max_component_wise() {
    let a = Vec3::new(1.0, 5.0, -3.0);
    let b = Vec3::new(2.0, -1.0, 4.0);
    let m = Vec3::max(a, b);
    assert_eq!(m, Vec3::new(2.0, 5.0, 4.0));
}

#[test]
fn vec3_abs_all_positive() {
    let v = Vec3::new(-1.0, 2.0, -3.0);
    assert_eq!(v.abs(), Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn vec3_min_component_picks_min() {
    let v = Vec3::new(1.0, -5.0, 3.0);
    assert!((v.min_component() - (-5.0)).abs() < 1e-6);
}

#[test]
fn vec3_max_component_picks_max() {
    let v = Vec3::new(1.0, -5.0, 3.0);
    assert!((v.max_component() - 3.0).abs() < 1e-6);
}

#[test]
fn vec3_neg_flips_sign() {
    let v = Vec3::new(1.0, -2.0, 3.0);
    assert_eq!(-v, Vec3::new(-1.0, 2.0, -3.0));
}

#[test]
fn quat_from_rotation_between_same_vector() {
    let a = Vec3::new(1.0, 0.0, 0.0);
    let q = Quat::from_rotation_between(a, a);
    // 同向量 → 单位四元数 (w ≈ 1, xyz ≈ 0)
    let v = q.rotate_vec3(a);
    assert!((v - a).length() < 1e-5);
}

#[test]
fn quat_from_rotation_between_perpendicular_90deg() {
    let from = Vec3::new(1.0, 0.0, 0.0);
    let to = Vec3::new(0.0, 1.0, 0.0);
    let q = Quat::from_rotation_between(from, to);
    let rotated = q.rotate_vec3(from);
    assert!((rotated - to).length() < 1e-5);
}

#[test]
fn quat_look_rotation_aligned_with_forward() {
    // look_rotation(forward=+Z, up=+Y) → 旋转 +Z 到 +Z，identity
    let q = Quat::look_rotation(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
    let fwd = q.rotate_vec3(Vec3::new(0.0, 0.0, 1.0));
    assert!((fwd - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-5);
}

#[test]
fn aabb_intersection_overlap_returns_smaller_box() {
    let a = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
    let b = Aabb::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(3.0, 3.0, 3.0));
    let inter = a.intersection(b).expect("overlap exists");
    assert_eq!(inter.min, Vec3::new(1.0, 1.0, 1.0));
    assert_eq!(inter.max, Vec3::new(2.0, 2.0, 2.0));
}

#[test]
fn aabb_intersection_disjoint_returns_none() {
    let a = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
    let b = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    assert!(a.intersection(b).is_none());
}

#[test]
fn transform_lerp_midpoint() {
    let a = Transform::new(Vec3::ZERO, Quat::IDENTITY);
    let b = Transform::new(
        Vec3::new(2.0, 4.0, 6.0),
        Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI),
    );
    let mid = Transform::lerp(a, b, 0.5);
    // translation lerp at t=0.5 = (1, 2, 3)
    assert!((mid.translation.x - 1.0).abs() < 1e-5);
    assert!((mid.translation.y - 2.0).abs() < 1e-5);
    assert!((mid.translation.z - 3.0).abs() < 1e-5);
}

// ============================================================================
// 属性测试 (proptest!)
// ============================================================================

proptest! {
    // ---- Category 1: 四元数 (Quat) 不变量 ----

    /// 单位四元数：`normalize` 后 `||q|| = 1`。
    ///
    /// 构造任意非零四元数，normalize 后长度应严格 = 1（± f32 精度）。
    #[test]
    fn quat_normalize_yields_unit(
        x in -1.0_f32..1.0,
        y in -1.0_f32..1.0,
        z in -1.0_f32..1.0,
        w in -1.0_f32..1.0,
    ) {
        let q = Quat::new(x, y, z, w);
        // 跳过近零向量：normalize 走 fallback (返回原值) 路径
        prop_assume!(q.length_squared() > 1e-6);
        let n = q.normalize();
        prop_assert!(
            (n.length() - 1.0).abs() < 1e-5,
            "normalize 后长度应=1，实际={}",
            n.length()
        );
        // 二次 normalize 是 no-op（已单位化）
        let n2 = n.normalize();
        prop_assert!((n2.length() - 1.0).abs() < 1e-5);
    }

    /// 四元数乘法结合性：`(a*b)*c ≈ a*(b*c)`。
    ///
    /// 使用 axis-angle 构造的单位四元数（限角度 ±π 避免 2π 模糊）。
    #[test]
    fn quat_mul_associative(
        ax in -1.0_f32..1.0, ay in -1.0_f32..1.0, az in -1.0_f32..1.0,
        bx in -1.0_f32..1.0, by in -1.0_f32..1.0, bz in -1.0_f32..1.0,
        cx in -1.0_f32..1.0, cy in -1.0_f32..1.0, cz in -1.0_f32..1.0,
        // 用 std::f32::consts::PI 而不是 3.14 — clippy::approx_constant (deny 级)
        // 拒绝近似常量; 区间语义不变 (原本就是用 ±π 的近似值).
        ang_a in -std::f32::consts::PI..std::f32::consts::PI,
        ang_b in -std::f32::consts::PI..std::f32::consts::PI,
        ang_c in -std::f32::consts::PI..std::f32::consts::PI,
    ) {
        let axis_a = Vec3::new(ax, ay, az);
        let axis_b = Vec3::new(bx, by, bz);
        let axis_c = Vec3::new(cx, cy, cz);
        prop_assume!(axis_a.length_squared() > 1e-6);
        prop_assume!(axis_b.length_squared() > 1e-6);
        prop_assume!(axis_c.length_squared() > 1e-6);
        let a = Quat::from_axis_angle(axis_a.normalize(), ang_a);
        let b = Quat::from_axis_angle(axis_b.normalize(), ang_b);
        let c = Quat::from_axis_angle(axis_c.normalize(), ang_c);
        let left = a.mul(b).mul(c);
        let right = a.mul(b.mul(c));
        let diff = (left.x - right.x).abs()
            .max((left.y - right.y).abs())
            .max((left.z - right.z).abs())
            .max((left.w - right.w).abs());
        prop_assert!(
            diff < 1e-3,
            "四元数乘法不结合：max|差|={}",
            diff
        );
    }

    /// 四元数乘单位元：`q * IDENTITY ≈ q` 且 `IDENTITY * q ≈ q`。
    #[test]
    fn quat_identity_neutral(
        x in -1.0_f32..1.0,
        y in -1.0_f32..1.0,
        z in -1.0_f32..1.0,
        w in -1.0_f32..1.0,
    ) {
        let q = Quat::new(x, y, z, w);
        let r1 = q.mul(Quat::IDENTITY);
        let r2 = Quat::IDENTITY.mul(q);
        prop_assert!((r1.x - q.x).abs() < 1e-5, "q*I.x 偏差 {}", r1.x - q.x);
        prop_assert!((r1.y - q.y).abs() < 1e-5, "q*I.y 偏差 {}", r1.y - q.y);
        prop_assert!((r1.z - q.z).abs() < 1e-5, "q*I.z 偏差 {}", r1.z - q.z);
        prop_assert!((r1.w - q.w).abs() < 1e-5, "q*I.w 偏差 {}", r1.w - q.w);
        prop_assert_eq!(r1, r2, "q*I 与 I*q 应相等");
    }

    /// 旋转 4 元素 vs 旋转矩阵等价性：`quat.to_mat3().mul_vec3(v) ≈ quat.rotate_vec3(v)`。
    ///
    /// `Mat3::to_quat` 未暴露，所以无法做严格 quat→matrix→quat 闭环；这里验证
    /// "四元数旋转" 和 "矩阵旋转" 在任意向量上的语义等价（功能往返）。
    #[test]
    fn quat_to_mat3_matches_rotate_vec3(
        ax in -1.0_f32..1.0, ay in -1.0_f32..1.0, az in -1.0_f32..1.0,
        angle in -std::f32::consts::PI..std::f32::consts::PI,
        vx in -100.0_f32..100.0, vy in -100.0_f32..100.0, vz in -100.0_f32..100.0,
    ) {
        let axis = Vec3::new(ax, ay, az);
        prop_assume!(axis.length_squared() > 1e-6);
        let q = Quat::from_axis_angle(axis.normalize(), angle);
        let m = q.to_mat3();
        let v = Vec3::new(vx, vy, vz);
        let via_quat = q.rotate_vec3(v);
        let via_mat = m.mul_vec3(v);
        let diff = (via_quat - via_mat).length();
        prop_assert!(
            diff < 1e-3,
            "quat vs mat3 旋转结果不一致：|差|={}（quat={:?} mat={:?}）",
            diff, via_quat, via_mat
        );
    }

    // ---- Category 2: 旋转矩阵 (Mat3) 不变量 ----

    /// 正交性：`R * R^T ≈ I` 且 `det(R) ≈ 1`。
    ///
    /// 通过 axis-angle 构造单位四元数 → `to_mat3()` 得到旋转矩阵。
    /// det 通过 `a·(b×c)` 公式（行向量叉积）计算。
    #[test]
    fn mat3_from_quat_is_orthogonal(
        ax in -1.0_f32..1.0, ay in -1.0_f32..1.0, az in -1.0_f32..1.0,
        angle in -std::f32::consts::PI..std::f32::consts::PI,
    ) {
        let axis = Vec3::new(ax, ay, az);
        prop_assume!(axis.length_squared() > 1e-6);
        let q = Quat::from_axis_angle(axis.normalize(), angle);
        let r = q.to_mat3();
        let rt = r.transpose();
        let prod = r.mul_mat3(rt);
        // R * R^T 应 ≈ IDENTITY
        let err_row0 = (prod.m[0] - Vec3::new(1.0, 0.0, 0.0)).length();
        let err_row1 = (prod.m[1] - Vec3::new(0.0, 1.0, 0.0)).length();
        let err_row2 = (prod.m[2] - Vec3::new(0.0, 0.0, 1.0)).length();
        let total_err = err_row0 + err_row1 + err_row2;
        prop_assert!(
            total_err < 1e-3,
            "R*R^T != I：行误差和={}（行0={} 行1={} 行2={}）",
            total_err, err_row0, err_row1, err_row2
        );
        // det(R) = 1（行向量叉积再点积）
        let det = r.m[0].dot(r.m[1].cross(r.m[2]));
        prop_assert!(
            (det - 1.0).abs() < 1e-3,
            "det(R) != 1，实际={}",
            det
        );
    }

    /// 旋转矩阵作用于向量保持模长：`||R*v|| ≈ ||v||`。
    #[test]
    fn mat3_preserves_length(
        ax in -1.0_f32..1.0, ay in -1.0_f32..1.0, az in -1.0_f32..1.0,
        angle in -std::f32::consts::PI..std::f32::consts::PI,
        vx in -100.0_f32..100.0, vy in -100.0_f32..100.0, vz in -100.0_f32..100.0,
    ) {
        let axis = Vec3::new(ax, ay, az);
        prop_assume!(axis.length_squared() > 1e-6);
        let q = Quat::from_axis_angle(axis.normalize(), angle);
        let r = q.to_mat3();
        let v = Vec3::new(vx, vy, vz);
        let len_orig = v.length();
        let len_new = r.mul_vec3(v).length();
        // 相对误差：原长为 0 时用绝对误差
        if len_orig > 1e-6 {
            let rel = (len_orig - len_new).abs() / len_orig;
            prop_assert!(
                rel < 1e-3,
                "R*v 长度不一致：orig={} new={} 相对误差={}",
                len_orig, len_new, rel
            );
        } else {
            prop_assert!(len_new < 1e-3, "零向量旋转后长度应≈0，实际={}", len_new);
        }
    }

    /// 旋转矩阵列向量两两正交：`col_i · col_j ≈ δ_ij`。
    #[test]
    fn mat3_columns_are_orthonormal(
        ax in -1.0_f32..1.0, ay in -1.0_f32..1.0, az in -1.0_f32..1.0,
        angle in -std::f32::consts::PI..std::f32::consts::PI,
    ) {
        let axis = Vec3::new(ax, ay, az);
        prop_assume!(axis.length_squared() > 1e-6);
        let q = Quat::from_axis_angle(axis.normalize(), angle);
        let r = q.to_mat3();
        // 列向量：col_j = m[0].?j, m[1].?j, m[2].?j（from_basis 文档约定）
        let c0 = Vec3::new(r.m[0].x, r.m[1].x, r.m[2].x);
        let c1 = Vec3::new(r.m[0].y, r.m[1].y, r.m[2].y);
        let c2 = Vec3::new(r.m[0].z, r.m[1].z, r.m[2].z);
        // 单位长度
        prop_assert!((c0.length() - 1.0).abs() < 1e-4, "col0 长度 {}", c0.length());
        prop_assert!((c1.length() - 1.0).abs() < 1e-4, "col1 长度 {}", c1.length());
        prop_assert!((c2.length() - 1.0).abs() < 1e-4, "col2 长度 {}", c2.length());
        // 两两正交
        prop_assert!(c0.dot(c1).abs() < 1e-4, "col0·col1={}", c0.dot(c1));
        prop_assert!(c0.dot(c2).abs() < 1e-4, "col0·col2={}", c0.dot(c2));
        prop_assert!(c1.dot(c2).abs() < 1e-4, "col1·col2={}", c1.dot(c2));
    }

    // ---- Category 3: 投影不变量 ----

    /// 投影幂等性：`project(project(v, u), u) ≈ project(v, u)`。
    ///
    /// `project_onto` 返回的向量与 `u` 共线，再次投影应不变。
    #[test]
    fn project_onto_idempotent(
        vx in -100.0_f32..100.0, vy in -100.0_f32..100.0, vz in -100.0_f32..100.0,
        ux in -10.0_f32..10.0, uy in -10.0_f32..10.0, uz in -10.0_f32..10.0,
    ) {
        let v = Vec3::new(vx, vy, vz);
        let u = Vec3::new(ux, uy, uz);
        prop_assume!(u.length_squared() > 1e-6);
        let p1 = v.project_onto(u);
        let p2 = p1.project_onto(u);
        let diff = (p1 - p2).length();
        prop_assert!(
            diff < 1e-4,
            "project 不幂等：||p1 - p2||={}（p1={:?} p2={:?}）",
            diff, p1, p2
        );
    }

    /// 投影残差正交：`v - project(v, n) ⊥ n`（`n` 为单位向量）。
    ///
    /// 这等价于"投影到以 n 为法向的平面上，残差在该平面内"的几何意义。
    #[test]
    fn project_onto_residual_orthogonal(
        vx in -100.0_f32..100.0, vy in -100.0_f32..100.0, vz in -100.0_f32..100.0,
        nx in -1.0_f32..1.0, ny in -1.0_f32..1.0, nz in -1.0_f32..1.0,
    ) {
        let v = Vec3::new(vx, vy, vz);
        let n_raw = Vec3::new(nx, ny, nz);
        prop_assume!(n_raw.length_squared() > 1e-6);
        let n = n_raw.normalize(); // 单位法向
        let p = v.project_onto(n);
        let residual = v - p;
        let dot = residual.dot(n);
        // 残差长度在有限域 (|v|≤173, |p|≤173) 下，dot 误差受 f32 精度限制
        prop_assert!(
            dot.abs() < 1e-3,
            "残差不与法向正交：dot={}（residual={:?} n={:?}）",
            dot, residual, n
        );
        // 残差长度 ≤ 原向量长度（投影缩减值）
        prop_assert!(
            residual.length() <= v.length() + 1e-4,
            "残差长度应 ≤ |v|，实际 residual={} v={}",
            residual.length(), v.length()
        );
    }

    // ---- Category 4: 三角恒等式 ----

    /// `sin²x + cos²x = 1`：通过 `Quat::from_axis_angle(axis, θ)` 隐式验证。
    ///
    /// axis-angle 构造：`q = (axis·sin(θ/2), cos(θ/2))`，故 `|q|² = sin²(θ/2) + cos²(θ/2) = 1`。
    /// 因此"axis-angle 构造的四元数必须是单位四元数"等价于"sin²+cos²=1"的实现。
    #[test]
    fn axis_angle_quat_is_unit(
        ax in -1.0_f32..1.0, ay in -1.0_f32..1.0, az in -1.0_f32..1.0,
        angle in -std::f32::consts::PI..std::f32::consts::PI,
    ) {
        let axis = Vec3::new(ax, ay, az);
        prop_assume!(axis.length_squared() > 1e-6);
        let q = Quat::from_axis_angle(axis.normalize(), angle);
        let len_sq = q.length_squared();
        prop_assert!(
            (len_sq - 1.0).abs() < 1e-5,
            "axis-angle 四元数应为单位四元数：|q|²={}（angle={}）",
            len_sq, angle
        );
        // 进一步：q 与其共轭（=逆）的乘积应 ≈ IDENTITY
        let qinv = q.conjugate();
        let qqinv = q.mul(qinv);
        prop_assert!((qqinv.x).abs() < 1e-5, "q·q⁻¹.x={}", qqinv.x);
        prop_assert!((qqinv.y).abs() < 1e-5, "q·q⁻¹.y={}", qqinv.y);
        prop_assert!((qqinv.z).abs() < 1e-5, "q·q⁻¹.z={}", qqinv.z);
        prop_assert!((qqinv.w - 1.0).abs() < 1e-5, "q·q⁻¹.w={}", qqinv.w);
    }
}
