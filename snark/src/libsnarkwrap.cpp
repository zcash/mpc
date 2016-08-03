#include <sodium.h>
#include <iostream>
#include <stdexcept>
#include <assert.h>
#include "common/default_types/r1cs_ppzksnark_pp.hpp"
#include "algebra/curves/public_params.hpp"
#include "relations/arithmetic_programs/qap/qap.hpp"
#include "reductions/r1cs_to_qap/r1cs_to_qap.hpp"

using namespace std;
using namespace libsnark;

typedef default_r1cs_ppzksnark_pp curve_pp;
typedef default_r1cs_ppzksnark_pp::G1_type curve_G1;
typedef default_r1cs_ppzksnark_pp::G2_type curve_G2;
typedef default_r1cs_ppzksnark_pp::GT_type curve_GT;
typedef default_r1cs_ppzksnark_pp::Fp_type curve_Fr;

extern "C" void libsnarkwrap_init() {
    libsnark::inhibit_profiling_info = true;
    libsnark::inhibit_profiling_counters = true;
    assert(sodium_init() != -1);
    curve_pp::init_public_params();
}

// Fr

extern "C" curve_Fr libsnarkwrap_Fr_random() {
    return curve_Fr::random_element();
}

extern "C" curve_Fr libsnarkwrap_Fr_from(const char *a) {
    return curve_Fr(a);
}

extern "C" curve_Fr libsnarkwrap_Fr_add(curve_Fr *a, curve_Fr *b) {
    return *a + *b;
}

extern "C" curve_Fr libsnarkwrap_Fr_sub(curve_Fr *a, curve_Fr *b) {
    return *a - *b;
}

extern "C" curve_Fr libsnarkwrap_Fr_mul(curve_Fr *a, curve_Fr *b) {
    return *a * *b;
}

extern "C" curve_Fr libsnarkwrap_Fr_neg(curve_Fr *a) {
    return -(*a);
}

extern "C" bool libsnarkwrap_Fr_is_zero(curve_Fr *a) {
    return a->is_zero();
}

// G1

extern "C" curve_G1 libsnarkwrap_G1_zero() {
    return curve_G1::zero();
}

extern "C" curve_G1 libsnarkwrap_G1_one() {
    return curve_G1::one();
}

extern "C" curve_G1 libsnarkwrap_G1_random() {
    return curve_G1::random_element();
}

extern "C" bool libsnarkwrap_G1_is_zero(curve_G1 *p) {
    return p->is_zero();
}

extern "C" bool libsnarkwrap_G1_is_equal(curve_G1 *p, curve_G1 *q) {
    return *p == *q;
}

extern "C" curve_G1 libsnarkwrap_G1_add(curve_G1 *p, curve_G1 *q) {
    return *p + *q;
}

extern "C" curve_G1 libsnarkwrap_G1_sub(curve_G1 *p, curve_G1 *q) {
    return *p - *q;
}

extern "C" curve_G1 libsnarkwrap_G1_neg(curve_G1 *p) {
    return -(*p);
}

extern "C" curve_G1 libsnarkwrap_G1_scalarmul(curve_G1 *p, curve_Fr *q) {
    return (*q) * (*p);
}

// G2

extern "C" curve_G2 libsnarkwrap_G2_zero() {
    return curve_G2::zero();
}

extern "C" curve_G2 libsnarkwrap_G2_one() {
    return curve_G2::one();
}

extern "C" curve_G2 libsnarkwrap_G2_random() {
    return curve_G2::random_element();
}

extern "C" bool libsnarkwrap_G2_is_zero(curve_G2 *p) {
    return p->is_zero();
}

extern "C" bool libsnarkwrap_G2_is_equal(curve_G2 *p, curve_G2 *q) {
    return *p == *q;
}

extern "C" curve_G2 libsnarkwrap_G2_add(curve_G2 *p, curve_G2 *q) {
    return *p + *q;
}

extern "C" curve_G2 libsnarkwrap_G2_sub(curve_G2 *p, curve_G2 *q) {
    return *p - *q;
}

extern "C" curve_G2 libsnarkwrap_G2_neg(curve_G2 *p) {
    return -(*p);
}

extern "C" curve_G2 libsnarkwrap_G2_scalarmul(curve_G2 *p, curve_Fr *q) {
    return (*q) * (*p);
}

// Pairing

extern "C" curve_GT libsnarkwrap_gt_exp(curve_GT *p, curve_Fr *s) {
    return (*p) ^ (*s);
}

extern "C" curve_GT libsnarkwrap_pairing(curve_G1 *p, curve_G2 *q) {
    return curve_pp::reduced_pairing(*p, *q);
}
