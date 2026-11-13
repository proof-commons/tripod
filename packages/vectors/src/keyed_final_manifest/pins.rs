//! Review-time source identities; changing a pin requires explicit re-freeze.
use super::FrozenItem;

/// SHA-256 of the adoption's exact ASCII phrase, without framing.
pub const SECOND_KEY: [u8; 32] = [
    153, 141, 30, 87, 95, 140, 30, 217, 224, 80, 245, 125, 185, 183, 253, 45, 11, 135, 168, 61,
    126, 178, 233, 180, 249, 51, 102, 20, 229, 72, 75, 62,
];

type SourcePin = (FrozenItem, &'static str, &'static [u8], [u8; 32]);
pub(super) const SOURCES: &[SourcePin] = &[
    (
        FrozenItem::Declarations,
        "packages/architecture/src/keyed.rs",
        include_bytes!("../../../../packages/architecture/src/keyed.rs"),
        [
            137, 47, 199, 243, 186, 118, 214, 237, 107, 37, 247, 61, 247, 115, 27, 21, 159, 170,
            80, 243, 84, 194, 9, 203, 69, 200, 31, 74, 29, 9, 21, 116,
        ],
    ),
    (
        FrozenItem::Declarations,
        "packages/architecture/src/semantic.rs",
        include_bytes!("../../../../packages/architecture/src/semantic.rs"),
        [
            120, 165, 166, 100, 246, 194, 175, 190, 200, 88, 240, 31, 230, 154, 159, 162, 179, 71,
            213, 214, 63, 22, 121, 168, 1, 171, 18, 77, 119, 210, 183, 96,
        ],
    ),
    (
        FrozenItem::Realization,
        "packages/realization/src/keyed.rs",
        include_bytes!("../../../../packages/realization/src/keyed.rs"),
        [
            77, 25, 196, 97, 138, 82, 170, 145, 177, 213, 76, 46, 2, 254, 5, 169, 78, 97, 247, 120,
            58, 47, 200, 212, 246, 64, 181, 162, 67, 191, 100, 15,
        ],
    ),
    (
        FrozenItem::Realization,
        "packages/realization/src/declarations/keyed.rs",
        include_bytes!("../../../../packages/realization/src/declarations/keyed.rs"),
        [
            211, 163, 217, 180, 111, 5, 6, 198, 196, 195, 189, 51, 253, 172, 237, 229, 254, 77, 17,
            249, 176, 221, 19, 17, 77, 124, 74, 102, 235, 98, 1, 244,
        ],
    ),
    (
        FrozenItem::Realization,
        "packages/realization/src/relation.rs",
        include_bytes!("../../../../packages/realization/src/relation.rs"),
        [
            121, 227, 29, 7, 196, 138, 0, 208, 250, 22, 233, 155, 57, 157, 233, 233, 152, 24, 56,
            9, 229, 178, 226, 55, 53, 212, 69, 11, 106, 109, 170, 187,
        ],
    ),
    (
        FrozenItem::Plans,
        "packages/compiler/src/keyed_plan.rs",
        include_bytes!("../../../../packages/compiler/src/keyed_plan.rs"),
        [
            242, 169, 25, 123, 198, 175, 5, 117, 79, 117, 46, 226, 21, 121, 109, 148, 12, 240, 164,
            176, 31, 230, 56, 13, 124, 30, 101, 89, 49, 84, 21, 203,
        ],
    ),
    (
        FrozenItem::Plans,
        "packages/compiler/src/engine_keyed.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed.rs"),
        [
            73, 218, 4, 195, 100, 58, 121, 144, 118, 253, 29, 115, 136, 83, 216, 248, 238, 115,
            152, 133, 142, 212, 145, 83, 255, 181, 253, 59, 71, 182, 156, 243,
        ],
    ),
    (
        FrozenItem::Plans,
        "packages/compiler/src/engine_keyed/program.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed/program.rs"),
        [
            219, 134, 1, 169, 37, 45, 137, 249, 190, 226, 79, 87, 242, 240, 109, 89, 103, 204, 238,
            77, 145, 208, 236, 170, 164, 63, 176, 157, 134, 167, 63, 196,
        ],
    ),
    (
        FrozenItem::Plans,
        "packages/compiler/src/engine_keyed/program/refusal.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed/program/refusal.rs"),
        [
            41, 96, 214, 66, 114, 159, 149, 125, 234, 14, 30, 75, 42, 214, 159, 83, 130, 140, 78,
            32, 146, 5, 203, 155, 183, 116, 247, 75, 149, 123, 44, 80,
        ],
    ),
    (
        FrozenItem::Plans,
        "packages/compiler/src/engine_keyed/program/restore.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed/program/restore.rs"),
        [
            54, 48, 107, 124, 174, 173, 115, 216, 53, 117, 38, 110, 196, 240, 234, 148, 191, 123,
            13, 152, 243, 162, 185, 183, 14, 0, 254, 91, 123, 25, 159, 227,
        ],
    ),
    (
        FrozenItem::Plans,
        "packages/compiler/src/engine_keyed/program/stage_refusal.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed/program/stage_refusal.rs"),
        [
            19, 19, 90, 100, 118, 177, 152, 94, 138, 2, 149, 9, 255, 25, 113, 53, 151, 234, 66, 24,
            216, 205, 38, 223, 166, 98, 142, 82, 204, 191, 30, 170,
        ],
    ),
    (
        FrozenItem::Selection,
        "packages/vectors/src/keyed_final_selection.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_final_selection.rs"),
        [
            140, 248, 90, 8, 10, 184, 4, 30, 176, 22, 167, 11, 32, 221, 8, 104, 57, 124, 194, 168,
            88, 101, 246, 134, 6, 123, 115, 227, 137, 213, 115, 182,
        ],
    ),
    (
        FrozenItem::Grammar,
        "packages/tapscript/src/keyed_constructor.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_constructor.rs"),
        [
            155, 184, 179, 170, 152, 217, 252, 246, 203, 223, 201, 220, 91, 131, 149, 13, 14, 87,
            247, 112, 177, 182, 91, 144, 218, 94, 3, 203, 98, 107, 99, 174,
        ],
    ),
    (
        FrozenItem::Grammar,
        "packages/linker/src/keyed_bundle.rs",
        include_bytes!("../../../../packages/linker/src/keyed_bundle.rs"),
        [
            15, 215, 136, 139, 134, 140, 83, 9, 105, 64, 239, 103, 132, 215, 207, 44, 218, 75, 212,
            191, 20, 192, 202, 68, 41, 125, 32, 75, 171, 4, 102, 73,
        ],
    ),
    (
        FrozenItem::Grammar,
        "packages/tapscript/src/keyed_pattern.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_pattern.rs"),
        [
            29, 255, 219, 248, 11, 113, 173, 124, 105, 167, 65, 235, 241, 200, 80, 96, 9, 236, 121,
            177, 251, 60, 235, 71, 2, 197, 18, 94, 93, 0, 44, 59,
        ],
    ),
    (
        FrozenItem::Grammar,
        "packages/tapscript/src/keyed_bundle.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_bundle.rs"),
        [
            107, 215, 169, 167, 78, 190, 8, 159, 149, 37, 254, 161, 108, 238, 249, 18, 210, 115,
            167, 124, 174, 89, 116, 103, 234, 7, 98, 63, 89, 46, 197, 139,
        ],
    ),
    (
        FrozenItem::Recipe,
        "packages/transaction/src/keyed_taproot.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_taproot.rs"),
        [
            20, 104, 4, 156, 64, 193, 177, 234, 203, 169, 38, 16, 209, 189, 161, 200, 128, 169, 33,
            233, 24, 1, 184, 159, 206, 106, 196, 137, 150, 61, 90, 255,
        ],
    ),
    (
        FrozenItem::Recipe,
        "packages/transaction/src/keyed_curve.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_curve.rs"),
        [
            176, 16, 59, 115, 144, 212, 220, 4, 174, 244, 125, 95, 84, 163, 42, 230, 76, 140, 8,
            95, 136, 81, 80, 135, 86, 95, 163, 183, 243, 241, 230, 215,
        ],
    ),
    (
        FrozenItem::Recipe,
        "packages/transaction/src/taproot.rs",
        include_bytes!("../../../../packages/transaction/src/taproot.rs"),
        [
            196, 22, 181, 13, 154, 119, 92, 192, 55, 110, 140, 10, 251, 115, 237, 210, 81, 144,
            201, 138, 220, 183, 67, 20, 59, 9, 195, 166, 252, 171, 115, 223,
        ],
    ),
    (
        FrozenItem::Recipe,
        "packages/tapscript/src/state_constructor.rs",
        include_bytes!("../../../../packages/tapscript/src/state_constructor.rs"),
        [
            235, 84, 104, 219, 104, 223, 159, 216, 100, 165, 234, 75, 139, 103, 188, 43, 101, 61,
            21, 182, 7, 195, 105, 155, 205, 159, 135, 229, 126, 112, 68, 94,
        ],
    ),
    (
        FrozenItem::Target,
        "packages/target-elements/src/definition.rs",
        include_bytes!("../../../../packages/target-elements/src/definition.rs"),
        [
            20, 165, 172, 123, 171, 15, 110, 136, 93, 100, 9, 199, 118, 192, 17, 77, 55, 243, 128,
            255, 121, 233, 36, 36, 34, 124, 143, 115, 160, 152, 134, 192,
        ],
    ),
    (
        FrozenItem::Target,
        "packages/target-elements/src/encoding.rs",
        include_bytes!("../../../../packages/target-elements/src/encoding.rs"),
        [
            59, 240, 52, 30, 226, 209, 43, 4, 255, 178, 228, 18, 207, 125, 47, 242, 125, 75, 69,
            70, 201, 186, 83, 215, 215, 42, 65, 166, 88, 254, 180, 23,
        ],
    ),
    (
        FrozenItem::Target,
        "packages/target-elements/src/resource.rs",
        include_bytes!("../../../../packages/target-elements/src/resource.rs"),
        [
            237, 203, 54, 224, 73, 224, 81, 28, 200, 131, 40, 58, 106, 227, 42, 201, 7, 9, 126,
            246, 4, 254, 73, 212, 30, 14, 68, 18, 89, 224, 1, 147,
        ],
    ),
    (
        FrozenItem::Abi,
        "packages/transaction/src/keyed_abi.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_abi.rs"),
        [
            9, 80, 191, 168, 160, 206, 167, 228, 113, 250, 180, 59, 111, 91, 142, 239, 187, 88,
            179, 235, 148, 97, 80, 156, 164, 211, 250, 97, 42, 3, 150, 104,
        ],
    ),
    (
        FrozenItem::Abi,
        "packages/transaction/src/keyed_request.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_request.rs"),
        [
            33, 70, 66, 224, 54, 92, 142, 156, 126, 140, 174, 67, 61, 214, 112, 233, 155, 159, 24,
            104, 150, 29, 0, 149, 177, 10, 65, 247, 126, 94, 35, 11,
        ],
    ),
    (
        FrozenItem::Abi,
        "packages/transaction/src/keyed_inputs.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_inputs.rs"),
        [
            116, 182, 156, 22, 162, 53, 9, 40, 196, 185, 70, 39, 110, 192, 118, 23, 177, 218, 193,
            95, 134, 173, 22, 27, 8, 246, 174, 90, 133, 186, 71, 187,
        ],
    ),
    (
        FrozenItem::Abi,
        "packages/tapscript/src/keyed_members.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_members.rs"),
        [
            109, 82, 198, 125, 30, 146, 89, 22, 149, 231, 178, 38, 84, 85, 139, 202, 18, 239, 147,
            23, 58, 193, 59, 106, 111, 82, 79, 49, 139, 254, 26, 143,
        ],
    ),
    (
        FrozenItem::Dependencies,
        "Cargo.toml",
        include_bytes!("../../../../Cargo.toml"),
        [
            207, 49, 5, 177, 126, 211, 49, 179, 74, 216, 245, 158, 8, 115, 63, 200, 2, 149, 149,
            68, 64, 172, 68, 60, 23, 250, 82, 135, 133, 90, 127, 63,
        ],
    ),
    (
        FrozenItem::Dependencies,
        "packages/transaction/Cargo.toml",
        include_bytes!("../../../../packages/transaction/Cargo.toml"),
        [
            213, 48, 46, 59, 231, 172, 161, 133, 232, 101, 56, 158, 220, 252, 185, 8, 32, 120, 107,
            193, 224, 178, 123, 152, 160, 191, 18, 130, 92, 193, 86, 106,
        ],
    ),
    (
        FrozenItem::FundingSigning,
        "packages/transaction/src/keyed_construct.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_construct.rs"),
        [
            224, 238, 253, 37, 37, 184, 48, 74, 94, 225, 200, 197, 244, 43, 33, 12, 226, 47, 45,
            128, 129, 63, 75, 197, 215, 150, 164, 132, 40, 7, 232, 224,
        ],
    ),
    (
        FrozenItem::FundingSigning,
        "packages/transaction/src/keyed_sponsor.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_sponsor.rs"),
        [
            23, 197, 44, 193, 221, 223, 244, 186, 17, 137, 72, 96, 13, 113, 208, 37, 234, 225, 194,
            111, 194, 27, 87, 118, 68, 110, 85, 140, 28, 74, 172, 150,
        ],
    ),
    (
        FrozenItem::FundingSigning,
        "packages/transaction/src/construct.rs",
        include_bytes!("../../../../packages/transaction/src/construct.rs"),
        [
            18, 71, 217, 9, 166, 49, 40, 80, 41, 142, 72, 158, 168, 109, 160, 205, 199, 88, 42, 88,
            52, 38, 183, 59, 225, 184, 80, 85, 141, 198, 35, 42,
        ],
    ),
    (
        FrozenItem::PolicyEnvelope,
        "packages/target-elements/src/transaction_form.rs",
        include_bytes!("../../../../packages/target-elements/src/transaction_form.rs"),
        [
            136, 13, 145, 18, 134, 109, 117, 141, 78, 51, 179, 193, 96, 117, 131, 25, 143, 113,
            130, 116, 52, 14, 186, 131, 245, 79, 145, 12, 105, 237, 169, 166,
        ],
    ),
    (
        FrozenItem::PolicyEnvelope,
        "packages/vectors/src/keyed_scaling.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_scaling.rs"),
        [
            74, 33, 143, 80, 144, 149, 220, 19, 155, 159, 188, 18, 164, 92, 161, 126, 198, 75, 38,
            160, 75, 34, 151, 196, 71, 59, 123, 73, 147, 75, 167, 99,
        ],
    ),
    (
        FrozenItem::References,
        "packages/linker/src/keyed_reference.rs",
        include_bytes!("../../../../packages/linker/src/keyed_reference.rs"),
        [
            166, 232, 62, 249, 8, 166, 107, 188, 142, 12, 163, 131, 141, 77, 240, 67, 83, 74, 24,
            156, 184, 58, 46, 249, 14, 133, 222, 121, 7, 81, 167, 120,
        ],
    ),
];
