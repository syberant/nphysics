use na::{self, Isometry3, Matrix3, Real, Translation3, U3, UnitQuaternion, Vector3, VectorSlice3};

use utils::GeneralizedCross;
use joint::Joint;
use solver::IntegrationParameters;
use math::{JacobianSliceMut, Velocity};

#[derive(Copy, Clone, Debug)]
pub struct BallJoint<N: Real> {
    rot: UnitQuaternion<N>,

    jacobian_v: Matrix3<N>,
    jacobian_dot_v: Matrix3<N>,
}

impl<N: Real> BallJoint<N> {
    pub fn new(axisangle: Vector3<N>) -> Self {
        BallJoint {
            rot: UnitQuaternion::new(axisangle),
            jacobian_v: na::zero(),
            jacobian_dot_v: na::zero(),
        }
    }
}

impl<N: Real> Joint<N> for BallJoint<N> {
    #[inline]
    fn ndofs(&self) -> usize {
        3
    }

    fn body_to_parent(&self, parent_shift: &Vector3<N>, body_shift: &Vector3<N>) -> Isometry3<N> {
        let trans = Translation3::from_vector(parent_shift - self.rot * body_shift);
        Isometry3::from_parts(trans, self.rot)
    }

    fn update_jacobians(&mut self, body_shift: &Vector3<N>, vels: &[N]) {
        let shift = self.rot * -body_shift;
        let angvel = VectorSlice3::new(vels);

        self.jacobian_v = shift.gcross_matrix_tr();
        self.jacobian_dot_v = angvel.cross(&shift).gcross_matrix_tr();
    }

    fn jacobian(&self, transform: &Isometry3<N>, out: &mut JacobianSliceMut<N>) {
        // FIXME: could we avoid the computation of rotation matrix on each `jacobian_*`  ?
        let rotmat = transform.rotation.to_rotation_matrix();
        out.fixed_rows_mut::<U3>(0)
            .copy_from(&(rotmat * self.jacobian_v));
        out.fixed_rows_mut::<U3>(3).copy_from(rotmat.matrix());
    }

    fn jacobian_dot(&self, transform: &Isometry3<N>, out: &mut JacobianSliceMut<N>) {
        let rotmat = transform.rotation.to_rotation_matrix();
        out.fixed_rows_mut::<U3>(0)
            .copy_from(&(rotmat * self.jacobian_dot_v));
    }

    fn jacobian_dot_veldiff_mul_coordinates(
        &self,
        transform: &Isometry3<N>,
        acc: &[N],
        out: &mut JacobianSliceMut<N>,
    ) {
        let angvel = Vector3::from_row_slice(&acc[..3]);
        let rotmat = transform.rotation.to_rotation_matrix();
        let res = rotmat * angvel.gcross_matrix() * self.jacobian_v;
        out.fixed_rows_mut::<U3>(0).copy_from(&res);
    }

    fn jacobian_mul_coordinates(&self, acc: &[N]) -> Velocity<N> {
        let angvel = Vector3::from_row_slice(&acc[..3]);
        let linvel = self.jacobian_v * angvel;
        Velocity::new(linvel, angvel)
    }

    fn jacobian_dot_mul_coordinates(&self, acc: &[N]) -> Velocity<N> {
        let angvel = Vector3::from_row_slice(&acc[..3]);
        let linvel = self.jacobian_dot_v * angvel;
        Velocity::new(linvel, na::zero())
    }

    fn apply_displacement(&mut self, params: &IntegrationParameters<N>, vels: &[N]) {
        let angvel = Vector3::from_row_slice(&vels[..3]);
        let disp = UnitQuaternion::new(angvel * params.dt);
        self.rot = disp * self.rot;
    }
}
