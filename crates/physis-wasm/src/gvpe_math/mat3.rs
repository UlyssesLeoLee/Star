//! ═══════════════════════════════════════════════════════════════════
//! Ported from UlyssesLeoLee/Physis (MIT OR Apache-2.0) — GVPE 物理引擎内核
//! per 2026-11-14: 全 3D 主题 (Physis WASM + three.js 背景层)
//! Source: E:\Physis @ dev 3e2cf93 (gvpe-math/core/memory/dynamics 4 crate)
//! ═══════════════════════════════════════════════════════════════════

//! `Mat3`：3x3 矩阵（行优先）。

use bytemuck::{Pod, Zeroable};

/// 3x3 矩阵，行优先存储。
///
/// `[m11, m12, m13, m21, m22, m23, m31, m32, m33]` 即
/// ```text
/// | m11 m12 m13 |
/// | m21 m22 m23 |
/// | m31 m32 m33 |
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Mat3 {
    /// 3 行，每行是一个 `Vec3`（行优先存储）。
    pub m: [super::Vec3; 3],
}

// SAFETY: `Mat3` 仅含 `Vec3` 字段，标准布局，无 padding。
unsafe impl Pod for Mat3 {}
unsafe impl Zeroable for Mat3 {}

impl Mat3 {
    /// 零矩阵。
    pub const ZERO: Self = Self {
        m: [super::Vec3::ZERO; 3],
    };

    /// 单位矩阵。
    pub const IDENTITY: Self = Self {
        m: [
            super::Vec3::new(1.0, 0.0, 0.0),
            super::Vec3::new(0.0, 1.0, 0.0),
            super::Vec3::new(0.0, 0.0, 1.0),
        ],
    };

    /// 构造新矩阵（行优先）。
    #[inline]
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        m11: f32,
        m12: f32,
        m13: f32,
        m21: f32,
        m22: f32,
        m23: f32,
        m31: f32,
        m32: f32,
        m33: f32,
    ) -> Self {
        Self {
            m: [
                super::Vec3::new(m11, m12, m13),
                super::Vec3::new(m21, m22, m23),
                super::Vec3::new(m31, m32, m33),
            ],
        }
    }

    /// 构造对角矩阵，非对角元素 = 0。
    ///
    /// 行优先存储：`m.m[0][0] = diag.x`，`m.m[1][1] = diag.y`，`m.m[2][2] = diag.z`。
    /// 满足 `m * Vec3(x, y, z) == Vec3(diag.x * x, diag.y * y, diag.z * z)`（逐分量缩放）。
    #[inline]
    #[must_use]
    pub const fn from_diagonal(diag: super::Vec3) -> Self {
        Self::new(diag.x, 0.0, 0.0, 0.0, diag.y, 0.0, 0.0, 0.0, diag.z)
    }

    /// 由三条基轴构造矩阵。
    ///
    /// **约定：列向量**（与 [`Mat3::mul_vec3`] 一致，即 `m * v = m.x * v.x + m.y * v.y + m.z * v.z`）。
    /// 行优先存储实现为：
    /// ```text
    /// | x.x y.x z.x |
    /// | x.y y.y z.y |
    /// | x.z y.z z.z |
    /// ```
    /// 调用方对输入是否归一化不假设；`x`、`y`、`z` 彼此正交时是标准旋转 / 坐标系矩阵。
    #[inline]
    #[must_use]
    pub const fn from_basis(x: super::Vec3, y: super::Vec3, z: super::Vec3) -> Self {
        Self::new(x.x, y.x, z.x, x.y, y.y, z.y, x.z, y.z, z.z)
    }

    /// 从单位四元数构造旋转矩阵。
    ///
    /// `q` 应为单位四元数；非单位四元数得到的是 scaled rotation（每行长度 = |q|²）。
    /// 矩阵布局与 `Mul<Vec3>` 一致：`M * v` 等价于 `q.rotate_vec3(v)`。
    #[allow(clippy::many_single_char_names)]
    #[inline]
    #[must_use]
    pub fn from_quat(q: super::Quat) -> Self {
        // 经典四元数→旋转矩阵公式（行主序）：
        //   r = 2 / |q|²
        //   M = [ 1 - r*(y² + z²),     r*(x*y - z*w),     r*(x*z + y*w) ]
        //       [     r*(x*y + z*w), 1 - r*(x² + z²),     r*(y*z - x*w) ]
        //       [     r*(x*z - y*w),     r*(y*z + x*w), 1 - r*(x² + y²) ]
        // 假设 |q|=1 故 r=2
        let (x, y, z, w) = (q.x, q.y, q.z, q.w);
        // 用 mul_add 减少算入：xx = 2*x², yy = 2*y², zz = 2*z²
        let xx = 2.0 * x * x;
        let yy = 2.0 * y * y;
        let zz = 2.0 * z * z;
        let xy = 2.0 * x * y;
        let xz = 2.0 * x * z;
        let yz = 2.0 * y * z;
        let wx = 2.0 * w * x;
        let wy = 2.0 * w * y;
        let wz = 2.0 * w * z;
        Self::new(
            1.0 - (yy + zz),
            xy - wz,
            xz + wy,
            xy + wz,
            1.0 - (xx + zz),
            yz - wx,
            xz - wy,
            yz + wx,
            1.0 - (xx + yy),
        )
    }

    /// 矩阵 × 向量。
    #[inline]
    #[must_use]
    pub fn mul_vec3(self, v: super::Vec3) -> super::Vec3 {
        super::Vec3::new(
            self.m[0]
                .x
                .mul_add(v.x, self.m[0].y.mul_add(v.y, self.m[0].z * v.z)),
            self.m[1]
                .x
                .mul_add(v.x, self.m[1].y.mul_add(v.y, self.m[1].z * v.z)),
            self.m[2]
                .x
                .mul_add(v.x, self.m[2].y.mul_add(v.y, self.m[2].z * v.z)),
        )
    }

    /// 矩阵 × 矩阵。
    ///
    /// 2026-11-21 修复: 原实现按 `self × rhs 的列` 组装结果, 但 `m` 是行优先
    /// 存储 (`m[i]` = 第 i 行), 导致乘积实际是 `(self × rhsᵀ)ᵀ` —— 对称矩阵
    /// (如单轴旋转) 恰好掩盖了错误, 非对称复合即暴露 (数学扩展 proptest
    /// mat3_mul_associative_on_vec 捕获)。正确语义: 结果第 i 行第 j 列 =
    /// self 第 i 行 · rhs 第 j 列。
    #[inline]
    #[must_use]
    pub fn mul_mat3(self, rhs: Self) -> Self {
        let c0 = super::Vec3::new(rhs.m[0].x, rhs.m[1].x, rhs.m[2].x);
        let c1 = super::Vec3::new(rhs.m[0].y, rhs.m[1].y, rhs.m[2].y);
        let c2 = super::Vec3::new(rhs.m[0].z, rhs.m[1].z, rhs.m[2].z);
        Self {
            m: [
                super::Vec3::new(self.m[0].dot(c0), self.m[0].dot(c1), self.m[0].dot(c2)),
                super::Vec3::new(self.m[1].dot(c0), self.m[1].dot(c1), self.m[1].dot(c2)),
                super::Vec3::new(self.m[2].dot(c0), self.m[2].dot(c1), self.m[2].dot(c2)),
            ],
        }
    }

    /// 转置。
    #[inline]
    #[must_use]
    pub const fn transpose(self) -> Self {
        Self::new(
            self.m[0].x,
            self.m[1].x,
            self.m[2].x,
            self.m[0].y,
            self.m[1].y,
            self.m[2].y,
            self.m[0].z,
            self.m[1].z,
            self.m[2].z,
        )
    }

    /// 求逆（伴随矩阵法 / 行列式）。
    ///
    /// 返回 `None` 表示矩阵奇异。
    #[allow(clippy::many_single_char_names)]
    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        let m = &self.m;
        let a = m[0].x;
        let b = m[0].y;
        let c = m[0].z;
        let d = m[1].x;
        let e = m[1].y;
        let f = m[1].z;
        let g = m[2].x;
        let h = m[2].y;
        let i = m[2].z;

        // 2026-11-21 重写: 原 mul_add 链式写法存在符号/参数序错误 (det 第三项
        // 与伴随矩阵多个余子式), 对角矩阵测试掩盖了问题。改为直白的余子式展开,
        // 由数学扩展 proptest (mat3_inverse_definition 等) 守护。
        // 余子式 (cofactor C_ij = (-1)^(i+j) * minor):
        let c00 = e * i - f * h;
        let c01 = -(d * i - f * g);
        let c02 = d * h - e * g;
        let c10 = -(b * i - c * h);
        let c11 = a * i - c * g;
        let c12 = -(a * h - b * g);
        let c20 = b * f - c * e;
        let c21 = -(a * f - c * d);
        let c22 = a * e - b * d;

        // det = 第一行展开
        let det = a * c00 + b * c01 + c * c02;
        if det.abs() < f32::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;

        // 逆 = 伴随 / det = 余子式矩阵的转置 / det
        Some(Self::new(
            c00 * inv_det,
            c10 * inv_det,
            c20 * inv_det,
            c01 * inv_det,
            c11 * inv_det,
            c21 * inv_det,
            c02 * inv_det,
            c12 * inv_det,
            c22 * inv_det,
        ))
    }
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}
