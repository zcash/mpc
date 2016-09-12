extern crate bn;
extern crate libc;
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

use bn::*;

extern "C" {
    fn libsnarkwrap_init();
    fn libsnarkwrap_getcs(d: *mut libc::uint64_t, vars: *mut libc::uint64_t, inputs: *mut libc::uint64_t, omega: *mut Fr) -> *mut libc::c_void;
    fn libsnarkwrap_dropcs(cs: *mut libc::c_void);
    fn libsnarkwrap_dropkeypair(kp: *mut libc::c_void);
    fn libsnarkwrap_eval(
        cs: *const libc::c_void,
        lc1: *const G1,
        lc2: *const G2,
        d: libc::uint64_t,
        vars: libc::uint64_t,
        at: *mut G1,
        bt1: *mut G1,
        bt2: *mut G2,
        ct: *mut G1);
    fn libsnarkwrap_construct_keypair(
        query_size: libc::uint64_t,
        pk_a: *const G1,
        pk_a_prime: *const G1,
        pk_b: *const G2,
        pk_b_prime: *const G1,
        pk_c: *const G1,
        pk_c_prime: *const G1,
        k_size: libc::uint64_t,
        pk_k: *const G1,
        h_size: libc::uint64_t,
        pk_h: *const G1,
        vk_a: *const G2,
        vk_b: *const G1,
        vk_c: *const G2,
        vk_gamma: *const G2,
        vk_beta_gamma_1: *const G1,
        vk_beta_gamma_2: *const G2,
        vk_z: *const G2,
        num_inputs: libc::uint64_t
    ) -> *mut libc::c_void;

    fn libsnarkwrap_keypair_eq(
        kp1: *const libc::c_void,
        kp2: *const libc::c_void,
    ) -> bool;
    fn libsnarkwrap_test_keygen(
        cs: *const libc::c_void,
        tau: *const Fr,
        alpha_a: *const Fr,
        alpha_b: *const Fr,
        alpha_c: *const Fr,
        rho_a: *const Fr,
        rho_b: *const Fr,
        beta: *const Fr,
        gamma: *const Fr
    ) -> *mut libc::c_void;
    fn libsnarkwrap_test_eval(
        cs: *const libc::c_void,
        tau: *const Fr,
        vars: libc::uint64_t,
        at: *const G1,
        bt1: *const G1,
        bt2: *const G2,
        ct: *const G1) -> bool;
    fn libsnarkwrap_test_compare_tau(
        i1: *const G1,
        i2: *const G2,
        tau: *const Fr,
        d: libc::uint64_t,
        qap: *const libc::c_void) -> bool;
}

lazy_static! {
    static ref INIT_LOCK: Mutex<bool> = Mutex::new(false);
}

/// This must be called before anything in this module is used.
fn initialize() {
    use std::mem::align_of;
    let mut l = INIT_LOCK.lock().unwrap();

    assert_eq!(align_of::<Fr>(), align_of::<libc::uint64_t>());
    assert_eq!(align_of::<G1>(), align_of::<libc::uint64_t>());
    assert_eq!(align_of::<G2>(), align_of::<libc::uint64_t>());
    assert_eq!(align_of::<Gt>(), align_of::<libc::uint64_t>());

    if !*l {
        unsafe { libsnarkwrap_init(); }
        *l = true;
    }
}

pub struct CS {
    ptr: *mut libc::c_void,
    pub d: usize,
    pub num_vars: usize,
    pub num_inputs: usize,
    pub omega: Fr
}

pub struct Keypair {
    ptr: *mut libc::c_void
}

impl PartialEq for Keypair {
    fn eq(&self, other: &Keypair) -> bool {
        initialize();

        unsafe {
            libsnarkwrap_keypair_eq(self.ptr, other.ptr)
        }
    }
}

impl Keypair {
    pub fn from(
        cs: &CS,
        pk_a: &[G1],
        pk_a_prime: &[G1],
        pk_b: &[G2],
        pk_b_prime: &[G1],
        pk_c: &[G1],
        pk_c_prime: &[G1],
        pk_k: &[G1],
        pk_h: &[G1],
        vk_a: &G2,
        vk_b: &G1,
        vk_c: &G2,
        vk_gamma: &G2,
        vk_beta_gamma_1: &G1,
        vk_beta_gamma_2: &G2,
        vk_z: &G2
    ) -> Keypair
    {
        initialize();

        assert_eq!(pk_a.len(), pk_a_prime.len());
        assert_eq!(pk_a.len(), pk_b.len());
        assert_eq!(pk_a.len(), pk_b_prime.len());
        assert_eq!(pk_a.len(), pk_c.len());
        assert_eq!(pk_a.len(), pk_c_prime.len());

        Keypair {
            ptr: unsafe {
                libsnarkwrap_construct_keypair(
                    pk_a.len() as u64,
                    &pk_a[0],
                    &pk_a_prime[0],
                    &pk_b[0],
                    &pk_b_prime[0],
                    &pk_c[0],
                    &pk_c_prime[0],
                    pk_k.len() as u64,
                    &pk_k[0],
                    pk_h.len() as u64,
                    &pk_h[0],
                    vk_a,
                    vk_b,
                    vk_c,
                    vk_gamma,
                    vk_beta_gamma_1,
                    vk_beta_gamma_2,
                    vk_z,
                    cs.num_inputs as u64
                )
            }
        }
    }

    pub fn generate(
        cs: &CS,
        tau: &Fr,
        alpha_a: &Fr,
        alpha_b: &Fr,
        alpha_c: &Fr,
        rho_a: &Fr,
        rho_b: &Fr,
        beta: &Fr,
        gamma: &Fr
    ) -> Keypair {
        initialize();

        unsafe {
            Keypair {
                ptr: libsnarkwrap_test_keygen(
                    cs.ptr, tau, alpha_a, alpha_b, alpha_c, rho_a, rho_b, beta, gamma
                )
            }
        }
    }
}

impl CS {
    pub fn dummy() -> Self {
        initialize();

        let mut d = 0;
        let mut vars = 0;
        let mut num_inputs = 0;
        let mut o = Fr::zero();

        let cs = unsafe { libsnarkwrap_getcs(&mut d, &mut vars, &mut num_inputs, &mut o) };

        CS {
            ptr: cs,
            num_vars: vars as usize,
            num_inputs: num_inputs as usize,
            d: d as usize,
            omega: o
        }
    }

    pub fn test_compare_tau(&self, v1: &[G1], v2: &[G2], tau: &Fr) -> bool {
        initialize();

        assert_eq!(v1.len(), v2.len());
        unsafe { libsnarkwrap_test_compare_tau(&v1[0], &v2[0], tau, v1.len() as u64, self.ptr) }
    }

    pub fn test_eval(&self, tau: &Fr, at: &[G1], bt1: &[G1], bt2: &[G2], ct: &[G1]) -> bool {
        initialize();

        assert_eq!(at.len(), bt1.len());
        assert_eq!(bt1.len(), bt2.len());
        assert_eq!(bt2.len(), ct.len());

        unsafe {
            libsnarkwrap_test_eval(self.ptr,
                                   tau,
                                   at.len() as u64,
                                   &at[0],
                                   &bt1[0],
                                   &bt2[0],
                                   &ct[0])
        }
    }

    pub fn eval(
        &self,
        lt1: &[G1],
        lt2: &[G2],
        at: &mut [G1],
        bt1: &mut [G1],
        bt2: &mut [G2],
        ct: &mut [G1]
    )
    {
        initialize();

        assert_eq!(lt1.len(), lt2.len());
        assert_eq!(at.len(), bt1.len());
        assert_eq!(bt1.len(), bt2.len());
        assert_eq!(bt2.len(), ct.len());

        unsafe {
            libsnarkwrap_eval(self.ptr,
                              &lt1[0],
                              &lt2[0],
                              lt1.len() as u64,
                              at.len() as u64,
                              &mut at[0],
                              &mut bt1[0],
                              &mut bt2[0],
                              &mut ct[0]);
        }
    }
}

impl Drop for CS {
    fn drop(&mut self) {
        initialize();

        unsafe { libsnarkwrap_dropcs(self.ptr) }
    }
}

impl Drop for Keypair {
    fn drop(&mut self) {
        initialize();
        
        unsafe { libsnarkwrap_dropkeypair(self.ptr) }
    }
}
