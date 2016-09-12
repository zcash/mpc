#include <sodium.h>
#include <iostream>
#include <stdexcept>
#include <assert.h>
#include "common/default_types/r1cs_ppzksnark_pp.hpp"
#include "algebra/curves/public_params.hpp"
#include "relations/arithmetic_programs/qap/qap.hpp"
#include "reductions/r1cs_to_qap/r1cs_to_qap.hpp"
#include "relations/constraint_satisfaction_problems/r1cs/examples/r1cs_examples.hpp"
#include "zk_proof_systems/ppzksnark/r1cs_ppzksnark/r1cs_ppzksnark.hpp"

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

    // Rust wrappers assume these sizes
    assert(sizeof(curve_Fr) == 8 * (4));
    assert(sizeof(curve_G1) == 8 * (4 * 3));
    assert(sizeof(curve_G2) == 8 * (4 * 2 * 3));
    assert(sizeof(curve_GT) == 8 * (4 * 6 * 2));

    // Rust wrappers assume alignment.
    assert(alignof(curve_Fr) == alignof(uint64_t));
    assert(alignof(curve_G1) == alignof(uint64_t));
    assert(alignof(curve_G2) == alignof(uint64_t));
    assert(alignof(curve_GT) == alignof(uint64_t));
}

// QAP

extern "C" void* libsnarkwrap_getcs(uint64_t *d, uint64_t *vars, uint64_t *num_inputs, curve_Fr *omega)
{
    // Generate a dummy circuit
    auto example = generate_r1cs_example_with_field_input<curve_Fr>(250, 4);

    // A/B swap
    example.constraint_system.swap_AB_if_beneficial();

    {
        // QAP reduction
        auto qap = r1cs_to_qap_instance_map(example.constraint_system);

        // Sanity checks
        assert(qap.A_in_Lagrange_basis.size() == example.constraint_system.num_variables()+1);
        assert(qap.B_in_Lagrange_basis.size() == example.constraint_system.num_variables()+1);
        assert(qap.C_in_Lagrange_basis.size() == example.constraint_system.num_variables()+1);

        // Degree of the QAP must be a power of 2
        assert(qap.degree() == 256);
        
        // Assume radix2 evaluation domain
        *omega = std::static_pointer_cast<basic_radix2_domain<curve_Fr>>(qap.domain)->omega;

        *d = qap.degree();
        *vars = example.constraint_system.num_variables()+1;
        *num_inputs = example.constraint_system.num_inputs();
    }
    
    return new r1cs_constraint_system<curve_Fr>(example.constraint_system);
}

extern "C" void libsnarkwrap_dropcs(r1cs_constraint_system<curve_Fr> *cs)
{
    delete cs;
}

extern "C" void libsnarkwrap_dropkeypair(r1cs_ppzksnark_keypair<curve_pp> *kp)
{
    delete kp;
}

extern "C" void libsnarkwrap_eval(
    const r1cs_constraint_system<curve_Fr> *cs,
    const curve_G1 *lc1,
    const curve_G2 *lc2,
    uint64_t d,
    uint64_t vars,
    curve_G1 *At,
    curve_G1 *Bt1,
    curve_G2 *Bt2,
    curve_G1 *Ct
)
{
    auto qap = r1cs_to_qap_instance_map(*cs);
    assert(qap.degree() == d);
    assert(qap.A_in_Lagrange_basis.size() == vars);
    assert(qap.B_in_Lagrange_basis.size() == vars);
    assert(qap.C_in_Lagrange_basis.size() == vars);

    for (size_t i = 0; i < vars; i++) {
        for (auto const &it : qap.A_in_Lagrange_basis[i]) {
            assert(it.first < d);
            At[i] = At[i] + it.second * lc1[it.first];
        }

        for (auto const &it : qap.B_in_Lagrange_basis[i]) {
            assert(it.first < d);
            Bt1[i] = Bt1[i] + it.second * lc1[it.first];
            Bt2[i] = Bt2[i] + it.second * lc2[it.first];
        }

        for (auto const &it : qap.C_in_Lagrange_basis[i]) {
            assert(it.first < d);
            Ct[i] = Ct[i] + it.second * lc1[it.first];
        }
    }
}

extern "C" void* libsnarkwrap_construct_keypair(
    uint64_t query_size,
    const curve_G1 *pk_a,
    const curve_G1 *pk_a_prime,
    const curve_G2 *pk_b,
    const curve_G1 *pk_b_prime,
    const curve_G1 *pk_c,
    const curve_G1 *pk_c_prime,
    uint64_t k_size,
    const curve_G1 *pk_k,
    uint64_t h_size,
    const curve_G1 *pk_h,
    const curve_G2 *vk_a,
    const curve_G1 *vk_b,
    const curve_G2 *vk_c,
    const curve_G2 *vk_gamma,
    const curve_G1 *vk_beta_gamma_1,
    const curve_G2 *vk_beta_gamma_2,
    const curve_G2 *vk_z,
    uint64_t num_inputs
)
{
    assert(query_size > num_inputs+1);
    auto keypair = new r1cs_ppzksnark_keypair<curve_pp>();

    // Construct proving key
    for (uint64_t i = 0; i < query_size; i++) {
        knowledge_commitment<curve_G1, curve_G1> cm_a(pk_a[i], pk_a_prime[i]);
        knowledge_commitment<curve_G2, curve_G1> cm_b(pk_b[i], pk_b_prime[i]);
        knowledge_commitment<curve_G1, curve_G1> cm_c(pk_c[i], pk_c_prime[i]);

        if (!cm_a.is_zero() && i > num_inputs) {
            keypair->pk.A_query.values.push_back(cm_a);
            keypair->pk.A_query.indices.push_back(i);
        }
        keypair->pk.A_query.domain_size_++;

        if (!cm_b.is_zero()) {
            keypair->pk.B_query.values.push_back(cm_b);
            keypair->pk.B_query.indices.push_back(i);
        }
        keypair->pk.B_query.domain_size_++;

        if (!cm_c.is_zero()) {
            keypair->pk.C_query.values.push_back(cm_c);
            keypair->pk.C_query.indices.push_back(i);
        }
        keypair->pk.C_query.domain_size_++;
    }

    for (uint64_t i = 0; i < k_size; i++) {
        keypair->pk.K_query.push_back(pk_k[i]);
    }

    for (uint64_t i = 0; i < h_size; i++) {
        keypair->pk.H_query.push_back(pk_h[i]);
    }

    // Construct verification key
    std::vector<curve_G1> IC_values;
    for (uint64_t i = 1; i < num_inputs+1; i++) {
        IC_values.push_back(pk_a[i]);
    }
    auto ic_base = pk_a[0];
    keypair->vk.encoded_IC_query = accumulation_vector<curve_G1 >(std::move(ic_base), std::move(IC_values));

    keypair->vk.alphaA_g2 = *vk_a;
    keypair->vk.alphaB_g1 = *vk_b;
    keypair->vk.alphaC_g2 = *vk_c;
    keypair->vk.gamma_g2 = *vk_gamma;
    keypair->vk.gamma_beta_g1 = *vk_beta_gamma_1;
    keypair->vk.gamma_beta_g2 = *vk_beta_gamma_2;
    keypair->vk.rC_Z_g2 = *vk_z;

    return keypair;
}

// Comparison tests

extern "C" void* libsnarkwrap_test_keygen(
    const r1cs_constraint_system<curve_Fr> *cs,
    const curve_Fr *tau,
    const curve_Fr *alpha_A,
    const curve_Fr *alpha_B,
    const curve_Fr *alpha_C,
    const curve_Fr *rho_A,
    const curve_Fr *rho_B,
    const curve_Fr *beta,
    const curve_Fr *gamma
)
{
    return new r1cs_ppzksnark_keypair<curve_pp>(
        r1cs_ppzksnark_generator<curve_pp>(
            *cs,
            *tau,
            *alpha_A,
            *alpha_B,
            *alpha_C,
            *rho_A,
            *rho_B,
            *beta,
            *gamma
        )
    );
}

extern "C" bool libsnarkwrap_keypair_eq(
    const r1cs_ppzksnark_keypair<curve_pp> *kp1,
    const r1cs_ppzksnark_keypair<curve_pp> *kp2
)
{
    std::string first_key;
    std::string second_key;
    {
        std::stringstream ss;
        ss << kp1->vk;
        ss << kp1->pk;
        first_key = ss.str();
    }
    {
        std::stringstream ss;
        ss << kp2->vk;
        ss << kp2->pk;
        second_key = ss.str();
    }

    return first_key == second_key;
}

extern "C" bool libsnarkwrap_test_compare_tau(
    const curve_G1 *inputs1,
    const curve_G2 *inputs2,
    const curve_Fr *tau,
    uint64_t d,
    const r1cs_constraint_system<curve_Fr> *cs
)
{
    auto qap = r1cs_to_qap_instance_map(*cs);
    auto coeffs = qap.domain->lagrange_coeffs(*tau);
    assert(coeffs.size() == d);
    assert(qap.degree() == d);

    bool res = true;
    for (size_t i = 0; i < d; i++) {
        res &= (coeffs[i] * curve_G1::one()) == inputs1[i];
        res &= (coeffs[i] * curve_G2::one()) == inputs2[i];
    }

    return res;
}

extern "C" bool libsnarkwrap_test_eval(
    const r1cs_constraint_system<curve_Fr> *cs,
    const curve_Fr *tau,
    uint64_t vars,
    const curve_G1 *At,
    const curve_G1 *Bt1,
    const curve_G2 *Bt2,
    const curve_G1 *Ct
) {
    auto qap = r1cs_to_qap_instance_map_with_evaluation(*cs, *tau);
    assert(qap.At.size() == vars);
    assert(qap.Bt.size() == vars);
    assert(qap.Ct.size() == vars);

    bool res = true;

    for (size_t i = 0; i < vars; i++) {
        res &= (qap.At[i] * curve_G1::one()) == At[i];
    }

    for (size_t i = 0; i < vars; i++) {
        res &= (qap.Bt[i] * curve_G1::one()) == Bt1[i];
        res &= (qap.Bt[i] * curve_G2::one()) == Bt2[i];
    }

    for (size_t i = 0; i < vars; i++) {
        res &= (qap.Ct[i] * curve_G1::one()) == Ct[i];
    }

    return res;
}
