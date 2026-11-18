//! S8 reviewed whole-file pins. The pin table represents itself through the
//! encoded inventory and is never hashed into itself.

pub(super) const SOURCES: &[(&str, &[u8], [u8; 32])] = &[
    (
        "Cargo.toml",
        include_bytes!("../../../../Cargo.toml"),
        [
            159, 54, 98, 67, 237, 234, 207, 116, 198, 107, 188, 202, 219, 223, 151, 199, 23, 79,
            78, 98, 211, 61, 223, 80, 101, 200, 155, 236, 95, 57, 116, 66,
        ],
    ),
    (
        "packages/architecture/Cargo.toml",
        include_bytes!("../../../../packages/architecture/Cargo.toml"),
        [
            168, 139, 239, 152, 98, 81, 200, 159, 198, 174, 195, 45, 78, 47, 239, 157, 167, 82, 34,
            56, 140, 147, 221, 109, 22, 165, 144, 187, 97, 176, 189, 211,
        ],
    ),
    (
        "packages/architecture/src/keyed_fixed.rs",
        include_bytes!("../../../../packages/architecture/src/keyed_fixed.rs"),
        [
            21, 172, 192, 219, 246, 237, 152, 204, 253, 126, 36, 128, 102, 80, 52, 119, 140, 220,
            155, 233, 50, 244, 69, 155, 44, 148, 8, 186, 252, 149, 242, 133,
        ],
    ),
    (
        "packages/architecture/src/keyed_fixed/vocabulary.rs",
        include_bytes!("../../../../packages/architecture/src/keyed_fixed/vocabulary.rs"),
        [
            42, 38, 196, 45, 24, 91, 151, 198, 160, 255, 9, 133, 9, 139, 56, 19, 101, 178, 234,
            122, 251, 4, 0, 3, 156, 156, 172, 234, 92, 203, 4, 27,
        ],
    ),
    (
        "packages/architecture/src/semantic.rs",
        include_bytes!("../../../../packages/architecture/src/semantic.rs"),
        [
            221, 40, 250, 100, 110, 219, 0, 72, 202, 196, 219, 104, 194, 16, 34, 102, 179, 39, 97,
            247, 20, 171, 4, 159, 254, 205, 126, 105, 11, 225, 211, 58,
        ],
    ),
    (
        "packages/compiler/Cargo.toml",
        include_bytes!("../../../../packages/compiler/Cargo.toml"),
        [
            122, 197, 236, 14, 83, 224, 30, 38, 45, 46, 198, 252, 18, 121, 228, 227, 20, 88, 174,
            180, 239, 64, 59, 237, 112, 13, 239, 14, 63, 242, 237, 18,
        ],
    ),
    (
        "packages/compiler/src/engine_keyed_fixed.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed_fixed.rs"),
        [
            138, 86, 122, 71, 152, 50, 116, 163, 126, 144, 210, 67, 255, 204, 123, 139, 164, 13,
            174, 150, 217, 94, 86, 202, 156, 96, 118, 60, 21, 129, 190, 117,
        ],
    ),
    (
        "packages/compiler/src/engine_keyed_fixed/program.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed_fixed/program.rs"),
        [
            6, 231, 135, 134, 177, 137, 95, 177, 118, 12, 10, 79, 8, 77, 191, 113, 205, 134, 55,
            51, 153, 116, 42, 101, 137, 211, 70, 187, 87, 181, 124, 37,
        ],
    ),
    (
        "packages/compiler/src/engine_keyed_fixed/program/refusal.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed_fixed/program/refusal.rs"),
        [
            40, 83, 93, 100, 45, 74, 116, 254, 150, 179, 197, 52, 234, 180, 86, 169, 47, 20, 199,
            90, 57, 200, 114, 229, 8, 104, 244, 51, 238, 225, 255, 227,
        ],
    ),
    (
        "packages/compiler/src/engine_keyed_fixed/program/restore.rs",
        include_bytes!("../../../../packages/compiler/src/engine_keyed_fixed/program/restore.rs"),
        [
            9, 176, 254, 222, 166, 42, 185, 84, 7, 239, 239, 226, 85, 156, 237, 218, 159, 196, 180,
            51, 227, 250, 242, 103, 119, 175, 213, 133, 28, 251, 219, 180,
        ],
    ),
    (
        "packages/compiler/src/engine_keyed_fixed/program/stage_refusal.rs",
        include_bytes!(
            "../../../../packages/compiler/src/engine_keyed_fixed/program/stage_refusal.rs"
        ),
        [
            223, 43, 177, 84, 219, 95, 94, 173, 58, 235, 82, 183, 230, 254, 173, 19, 22, 134, 140,
            182, 2, 213, 111, 171, 140, 97, 125, 113, 185, 161, 120, 134,
        ],
    ),
    (
        "packages/compiler/src/keyed_fixed_plan.rs",
        include_bytes!("../../../../packages/compiler/src/keyed_fixed_plan.rs"),
        [
            106, 251, 97, 230, 173, 111, 135, 152, 221, 238, 130, 162, 141, 170, 161, 106, 231,
            180, 227, 45, 31, 56, 136, 145, 184, 99, 35, 46, 119, 24, 221, 179,
        ],
    ),
    (
        "packages/linker/Cargo.toml",
        include_bytes!("../../../../packages/linker/Cargo.toml"),
        [
            162, 218, 78, 245, 190, 54, 130, 202, 42, 124, 36, 156, 82, 40, 174, 204, 43, 46, 166,
            75, 85, 180, 15, 150, 25, 236, 68, 80, 26, 239, 218, 102,
        ],
    ),
    (
        "packages/linker/src/keyed_fixed_bundle.rs",
        include_bytes!("../../../../packages/linker/src/keyed_fixed_bundle.rs"),
        [
            42, 37, 37, 70, 28, 221, 96, 195, 11, 54, 173, 247, 157, 22, 121, 105, 178, 31, 195,
            25, 90, 53, 32, 229, 28, 76, 114, 147, 92, 1, 215, 62,
        ],
    ),
    (
        "packages/linker/src/keyed_fixed_reference.rs",
        include_bytes!("../../../../packages/linker/src/keyed_fixed_reference.rs"),
        [
            63, 86, 174, 71, 226, 137, 41, 49, 190, 107, 188, 160, 177, 221, 186, 33, 149, 135,
            174, 128, 116, 253, 92, 159, 232, 189, 204, 125, 86, 101, 166, 223,
        ],
    ),
    (
        "packages/realization/Cargo.toml",
        include_bytes!("../../../../packages/realization/Cargo.toml"),
        [
            80, 28, 209, 186, 223, 242, 71, 36, 133, 172, 39, 93, 220, 255, 135, 225, 149, 64, 208,
            197, 250, 109, 47, 240, 57, 20, 10, 9, 30, 79, 5, 104,
        ],
    ),
    (
        "packages/realization/src/declarations/keyed_fixed.rs",
        include_bytes!("../../../../packages/realization/src/declarations/keyed_fixed.rs"),
        [
            215, 171, 226, 229, 30, 29, 133, 111, 255, 226, 178, 248, 75, 56, 245, 80, 2, 175, 135,
            235, 51, 37, 70, 222, 31, 120, 235, 200, 149, 117, 170, 135,
        ],
    ),
    (
        "packages/realization/src/keyed_fixed.rs",
        include_bytes!("../../../../packages/realization/src/keyed_fixed.rs"),
        [
            191, 53, 245, 2, 159, 222, 240, 56, 104, 205, 138, 102, 197, 2, 157, 78, 49, 121, 164,
            249, 187, 69, 137, 81, 52, 160, 236, 224, 72, 102, 242, 157,
        ],
    ),
    (
        "packages/realization/src/keyed_fixed_relation.rs",
        include_bytes!("../../../../packages/realization/src/keyed_fixed_relation.rs"),
        [
            42, 213, 40, 130, 53, 168, 107, 28, 85, 101, 24, 217, 144, 255, 53, 53, 174, 25, 9,
            123, 33, 141, 215, 105, 8, 60, 101, 40, 253, 214, 135, 28,
        ],
    ),
    (
        "packages/tapscript/Cargo.toml",
        include_bytes!("../../../../packages/tapscript/Cargo.toml"),
        [
            157, 236, 89, 208, 250, 215, 70, 3, 89, 104, 199, 162, 252, 150, 23, 229, 182, 28, 63,
            157, 149, 121, 143, 77, 45, 228, 174, 240, 19, 113, 107, 82,
        ],
    ),
    (
        "packages/tapscript/src/keyed_constructor.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_constructor.rs"),
        [
            155, 184, 179, 170, 152, 217, 252, 246, 203, 223, 201, 220, 91, 131, 149, 13, 14, 87,
            247, 112, 177, 182, 91, 144, 218, 94, 3, 203, 98, 107, 99, 174,
        ],
    ),
    (
        "packages/tapscript/src/keyed_fixed_bundle.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_fixed_bundle.rs"),
        [
            36, 140, 6, 209, 103, 102, 51, 25, 205, 117, 79, 166, 69, 224, 113, 52, 161, 251, 88,
            41, 138, 88, 221, 152, 211, 179, 65, 111, 9, 30, 36, 72,
        ],
    ),
    (
        "packages/tapscript/src/keyed_fixed_constructor.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_fixed_constructor.rs"),
        [
            64, 158, 31, 164, 57, 159, 145, 169, 150, 11, 148, 166, 243, 202, 103, 218, 192, 151,
            49, 71, 7, 166, 12, 164, 211, 235, 20, 113, 192, 19, 69, 207,
        ],
    ),
    (
        "packages/tapscript/src/keyed_fixed_members.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_fixed_members.rs"),
        [
            182, 76, 29, 20, 139, 14, 205, 207, 60, 119, 134, 47, 177, 61, 35, 198, 241, 81, 162,
            82, 178, 17, 9, 185, 173, 31, 92, 126, 133, 250, 82, 243,
        ],
    ),
    (
        "packages/tapscript/src/keyed_fixed_pattern.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_fixed_pattern.rs"),
        [
            51, 186, 214, 18, 29, 169, 207, 30, 153, 1, 169, 247, 17, 245, 116, 134, 118, 131, 8,
            68, 60, 9, 144, 104, 82, 151, 65, 213, 57, 55, 220, 154,
        ],
    ),
    (
        "packages/tapscript/src/keyed_fixed_resources.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_fixed_resources.rs"),
        [
            82, 42, 187, 254, 37, 223, 102, 32, 107, 90, 187, 65, 228, 152, 165, 195, 218, 83, 219,
            186, 13, 180, 107, 141, 138, 40, 4, 125, 18, 108, 5, 16,
        ],
    ),
    (
        "packages/tapscript/src/keyed_pattern.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_pattern.rs"),
        [
            29, 255, 219, 248, 11, 113, 173, 124, 105, 167, 65, 235, 241, 200, 80, 96, 9, 236, 121,
            177, 251, 60, 235, 71, 2, 197, 18, 94, 93, 0, 44, 59,
        ],
    ),
    (
        "packages/tapscript/src/keyed_resources.rs",
        include_bytes!("../../../../packages/tapscript/src/keyed_resources.rs"),
        [
            95, 46, 0, 146, 95, 50, 143, 139, 40, 147, 154, 103, 195, 111, 13, 23, 53, 214, 143,
            79, 108, 211, 206, 216, 132, 110, 49, 143, 216, 148, 201, 10,
        ],
    ),
    (
        "packages/tapscript/src/state_constructor.rs",
        include_bytes!("../../../../packages/tapscript/src/state_constructor.rs"),
        [
            235, 84, 104, 219, 104, 223, 159, 216, 100, 165, 234, 75, 139, 103, 188, 43, 101, 61,
            21, 182, 7, 195, 105, 155, 205, 159, 135, 229, 126, 112, 68, 94,
        ],
    ),
    (
        "packages/target-elements-conformance/Cargo.toml",
        include_bytes!("../../../../packages/target-elements-conformance/Cargo.toml"),
        [
            1, 194, 193, 2, 20, 143, 207, 34, 55, 189, 249, 65, 38, 204, 143, 89, 177, 92, 24, 127,
            42, 41, 38, 231, 22, 52, 90, 189, 165, 161, 210, 3,
        ],
    ),
    (
        "packages/target-elements-conformance/src/commitment_oracle/commitment.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/commitment_oracle/commitment.rs"
        ),
        [
            154, 182, 97, 141, 218, 94, 3, 38, 236, 202, 88, 189, 99, 203, 156, 91, 100, 104, 63,
            126, 12, 251, 120, 7, 162, 181, 66, 52, 246, 181, 104, 226,
        ],
    ),
    (
        "packages/target-elements-conformance/src/commitment_oracle/curve.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/commitment_oracle/curve.rs"
        ),
        [
            86, 27, 135, 54, 23, 20, 101, 234, 162, 119, 103, 199, 126, 224, 0, 35, 200, 29, 237,
            44, 169, 193, 239, 126, 203, 145, 234, 247, 162, 125, 112, 54,
        ],
    ),
    (
        "packages/target-elements-conformance/src/commitment_oracle/generator.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/commitment_oracle/generator.rs"
        ),
        [
            225, 239, 3, 211, 180, 205, 124, 168, 115, 157, 35, 247, 238, 39, 157, 215, 140, 42,
            100, 163, 92, 186, 175, 96, 214, 148, 187, 144, 246, 153, 128, 221,
        ],
    ),
    (
        "packages/target-elements-conformance/src/commitment_oracle/mod.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/commitment_oracle/mod.rs"
        ),
        [
            159, 5, 218, 32, 244, 71, 176, 12, 225, 175, 124, 167, 106, 218, 25, 163, 52, 155, 61,
            222, 0, 234, 19, 177, 252, 228, 165, 25, 198, 214, 179, 116,
        ],
    ),
    (
        "packages/target-elements-conformance/src/commitment_oracle/vector.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/commitment_oracle/vector.rs"
        ),
        [
            148, 140, 91, 193, 62, 60, 175, 188, 51, 67, 18, 234, 87, 109, 121, 119, 59, 161, 157,
            58, 39, 187, 29, 78, 110, 109, 211, 204, 165, 227, 130, 54,
        ],
    ),
    (
        "packages/target-elements-conformance/src/confidential_fixture.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/confidential_fixture.rs"
        ),
        [
            172, 42, 236, 182, 222, 36, 76, 214, 179, 25, 144, 233, 179, 230, 253, 55, 109, 148,
            151, 63, 238, 23, 116, 76, 206, 143, 27, 12, 127, 221, 16, 246,
        ],
    ),
    (
        "packages/target-elements-conformance/src/confidential_funding.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/confidential_funding.rs"
        ),
        [
            102, 230, 169, 77, 137, 73, 41, 76, 53, 108, 248, 4, 34, 85, 63, 39, 158, 51, 189, 64,
            104, 63, 3, 48, 207, 226, 167, 212, 105, 169, 124, 151,
        ],
    ),
    (
        "packages/target-elements-conformance/src/confidential_oracles.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/confidential_oracles.rs"
        ),
        [
            68, 88, 78, 107, 252, 85, 160, 70, 206, 239, 186, 251, 68, 247, 229, 188, 75, 140, 13,
            161, 237, 189, 188, 192, 143, 158, 28, 38, 75, 162, 19, 0,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/canonical.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/constructor/canonical.rs"
        ),
        [
            198, 154, 32, 155, 144, 39, 221, 142, 174, 168, 152, 194, 241, 11, 110, 173, 245, 93,
            214, 93, 241, 239, 208, 187, 176, 141, 220, 166, 197, 157, 160, 69,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/curve.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/constructor/curve.rs"),
        [
            197, 90, 58, 165, 105, 178, 193, 190, 29, 115, 38, 54, 238, 125, 3, 165, 80, 67, 175,
            89, 30, 32, 129, 250, 87, 195, 246, 177, 238, 244, 205, 81,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/internal_key.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/constructor/internal_key.rs"
        ),
        [
            199, 211, 214, 111, 78, 99, 69, 113, 209, 229, 170, 146, 231, 52, 210, 251, 132, 254,
            186, 129, 18, 145, 40, 162, 0, 169, 201, 99, 220, 17, 12, 194,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/metadata.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/constructor/metadata.rs"
        ),
        [
            214, 59, 192, 116, 65, 239, 250, 40, 126, 248, 235, 190, 190, 159, 227, 66, 113, 12,
            62, 254, 112, 95, 247, 135, 169, 239, 69, 9, 14, 91, 239, 141,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/metadata_leaf.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/constructor/metadata_leaf.rs"
        ),
        [
            211, 49, 214, 187, 44, 150, 71, 124, 220, 184, 207, 114, 47, 125, 163, 97, 151, 17,
            208, 51, 88, 239, 53, 138, 210, 196, 176, 26, 83, 202, 229, 223,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/mod.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/constructor/mod.rs"),
        [
            55, 72, 218, 75, 68, 159, 217, 209, 167, 197, 246, 103, 16, 56, 166, 242, 32, 92, 183,
            177, 104, 113, 40, 38, 31, 194, 95, 95, 79, 241, 115, 140,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/tagged.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/constructor/tagged.rs"
        ),
        [
            57, 6, 199, 200, 135, 170, 232, 60, 4, 93, 173, 63, 4, 210, 10, 128, 48, 200, 36, 207,
            44, 159, 212, 50, 214, 169, 231, 154, 120, 236, 225, 11,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/totality.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/constructor/totality.rs"
        ),
        [
            152, 1, 4, 12, 201, 97, 249, 136, 173, 58, 72, 1, 101, 46, 254, 232, 113, 42, 76, 118,
            194, 112, 46, 146, 86, 174, 48, 127, 155, 195, 181, 230,
        ],
    ),
    (
        "packages/target-elements-conformance/src/constructor/tree.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/constructor/tree.rs"),
        [
            54, 79, 12, 224, 130, 76, 135, 97, 22, 80, 145, 120, 108, 25, 85, 94, 182, 235, 68, 86,
            128, 42, 178, 39, 152, 91, 52, 150, 116, 148, 140, 34,
        ],
    ),
    (
        "packages/target-elements-conformance/src/error.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/error.rs"),
        [
            160, 98, 205, 247, 42, 72, 227, 218, 173, 26, 144, 170, 255, 24, 129, 2, 21, 21, 110,
            83, 59, 169, 64, 44, 120, 101, 136, 172, 200, 61, 84, 101,
        ],
    ),
    (
        "packages/target-elements-conformance/src/executor.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/executor.rs"),
        [
            225, 115, 177, 171, 86, 65, 29, 60, 99, 80, 126, 12, 251, 165, 178, 98, 1, 16, 176,
            238, 58, 170, 129, 208, 221, 161, 99, 3, 133, 202, 12, 63,
        ],
    ),
    (
        "packages/target-elements-conformance/src/fixture.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/fixture.rs"),
        [
            54, 203, 47, 113, 186, 139, 233, 46, 110, 83, 135, 115, 78, 1, 221, 70, 250, 58, 151,
            201, 118, 81, 39, 183, 23, 63, 203, 53, 179, 58, 206, 79,
        ],
    ),
    (
        "packages/target-elements-conformance/src/protocol.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/protocol.rs"),
        [
            143, 237, 241, 161, 202, 242, 132, 246, 113, 113, 214, 253, 61, 24, 70, 134, 71, 67,
            138, 173, 214, 129, 220, 85, 19, 184, 180, 97, 200, 198, 23, 37,
        ],
    ),
    (
        "packages/target-elements-conformance/src/prototype.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/prototype.rs"),
        [
            156, 219, 13, 27, 184, 238, 21, 92, 142, 238, 238, 16, 29, 223, 27, 52, 241, 221, 113,
            168, 38, 14, 173, 14, 105, 228, 35, 203, 114, 155, 165, 108,
        ],
    ),
    (
        "packages/target-elements-conformance/src/prototype_validate.rs",
        include_bytes!(
            "../../../../packages/target-elements-conformance/src/prototype_validate.rs"
        ),
        [
            80, 128, 191, 44, 12, 190, 166, 56, 184, 250, 64, 124, 4, 185, 244, 120, 83, 203, 122,
            238, 54, 239, 146, 237, 3, 224, 113, 51, 187, 103, 229, 231,
        ],
    ),
    (
        "packages/target-elements-conformance/src/provenance.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/provenance.rs"),
        [
            100, 223, 152, 191, 245, 123, 59, 164, 149, 12, 151, 173, 224, 249, 90, 89, 181, 114,
            207, 132, 46, 141, 151, 217, 236, 45, 211, 126, 129, 160, 244, 67,
        ],
    ),
    (
        "packages/target-elements-conformance/src/sponsor_parent.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/sponsor_parent.rs"),
        [
            44, 114, 212, 202, 5, 62, 42, 9, 117, 60, 214, 126, 185, 91, 177, 218, 31, 28, 213, 65,
            212, 170, 247, 60, 198, 80, 248, 108, 77, 39, 110, 172,
        ],
    ),
    (
        "packages/target-elements-conformance/src/test_material.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/test_material.rs"),
        [
            202, 37, 75, 6, 95, 44, 58, 129, 115, 187, 211, 56, 176, 177, 114, 143, 75, 139, 138,
            206, 199, 242, 221, 31, 26, 154, 12, 194, 116, 115, 145, 252,
        ],
    ),
    (
        "packages/target-elements-conformance/src/validate.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/validate.rs"),
        [
            243, 30, 116, 106, 139, 98, 144, 21, 46, 50, 168, 9, 76, 119, 248, 78, 139, 28, 19, 87,
            206, 23, 165, 9, 53, 252, 203, 239, 58, 202, 245, 122,
        ],
    ),
    (
        "packages/target-elements/Cargo.toml",
        include_bytes!("../../../../packages/target-elements/Cargo.toml"),
        [
            145, 203, 227, 80, 145, 46, 39, 239, 5, 209, 163, 230, 200, 142, 15, 88, 140, 123, 85,
            200, 104, 100, 159, 103, 218, 99, 180, 233, 43, 205, 58, 99,
        ],
    ),
    (
        "packages/target-elements/src/definition.rs",
        include_bytes!("../../../../packages/target-elements/src/definition.rs"),
        [
            20, 165, 172, 123, 171, 15, 110, 136, 93, 100, 9, 199, 118, 192, 17, 77, 55, 243, 128,
            255, 121, 233, 36, 36, 34, 124, 143, 115, 160, 152, 134, 192,
        ],
    ),
    (
        "packages/target-elements/src/encoding.rs",
        include_bytes!("../../../../packages/target-elements/src/encoding.rs"),
        [
            59, 240, 52, 30, 226, 209, 43, 4, 255, 178, 228, 18, 207, 125, 47, 242, 125, 75, 69,
            70, 201, 186, 83, 215, 215, 42, 65, 166, 88, 254, 180, 23,
        ],
    ),
    (
        "packages/target-elements/src/opcode.rs",
        include_bytes!("../../../../packages/target-elements/src/opcode.rs"),
        [
            150, 145, 101, 249, 176, 96, 179, 189, 60, 136, 150, 184, 101, 139, 184, 175, 114, 126,
            148, 143, 52, 87, 177, 89, 195, 57, 210, 225, 101, 4, 106, 24,
        ],
    ),
    (
        "packages/target-elements/src/resource.rs",
        include_bytes!("../../../../packages/target-elements/src/resource.rs"),
        [
            237, 203, 54, 224, 73, 224, 81, 28, 200, 131, 40, 58, 106, 227, 42, 201, 7, 9, 126,
            246, 4, 254, 73, 212, 30, 14, 68, 18, 89, 224, 1, 147,
        ],
    ),
    (
        "packages/target-elements/src/transaction_form.rs",
        include_bytes!("../../../../packages/target-elements/src/transaction_form.rs"),
        [
            136, 13, 145, 18, 134, 109, 117, 141, 78, 51, 179, 193, 96, 117, 131, 25, 143, 113,
            130, 116, 52, 14, 186, 131, 245, 79, 145, 12, 105, 237, 169, 166,
        ],
    ),
    (
        "packages/transaction/Cargo.toml",
        include_bytes!("../../../../packages/transaction/Cargo.toml"),
        [
            213, 48, 46, 59, 231, 172, 161, 133, 232, 101, 56, 158, 220, 252, 185, 8, 32, 120, 107,
            193, 224, 178, 123, 152, 160, 191, 18, 130, 92, 193, 86, 106,
        ],
    ),
    (
        "packages/transaction/src/bytes.rs",
        include_bytes!("../../../../packages/transaction/src/bytes.rs"),
        [
            227, 44, 4, 245, 220, 87, 139, 49, 169, 187, 176, 205, 148, 148, 217, 210, 250, 53,
            228, 113, 235, 76, 181, 82, 215, 112, 30, 52, 151, 24, 152, 153,
        ],
    ),
    (
        "packages/transaction/src/construct.rs",
        include_bytes!("../../../../packages/transaction/src/construct.rs"),
        [
            18, 71, 217, 9, 166, 49, 40, 80, 41, 142, 72, 158, 168, 109, 160, 205, 199, 88, 42, 88,
            52, 38, 183, 59, 225, 184, 80, 85, 141, 198, 35, 42,
        ],
    ),
    (
        "packages/transaction/src/keyed_curve.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_curve.rs"),
        [
            176, 16, 59, 115, 144, 212, 220, 4, 174, 244, 125, 95, 84, 163, 42, 230, 76, 140, 8,
            95, 136, 81, 80, 135, 86, 95, 163, 183, 243, 241, 230, 215,
        ],
    ),
    (
        "packages/transaction/src/keyed_fixed_abi.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_fixed_abi.rs"),
        [
            23, 209, 167, 123, 44, 192, 58, 78, 253, 106, 254, 76, 21, 53, 160, 227, 7, 80, 102,
            78, 21, 171, 63, 55, 228, 112, 128, 99, 15, 50, 172, 141,
        ],
    ),
    (
        "packages/transaction/src/keyed_fixed_construct.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_fixed_construct.rs"),
        [
            88, 117, 243, 88, 194, 223, 60, 37, 105, 166, 213, 106, 44, 186, 247, 225, 44, 2, 255,
            120, 67, 7, 128, 56, 175, 139, 157, 61, 101, 12, 57, 172,
        ],
    ),
    (
        "packages/transaction/src/keyed_fixed_inputs.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_fixed_inputs.rs"),
        [
            131, 128, 83, 83, 85, 253, 107, 55, 203, 38, 26, 199, 160, 58, 141, 176, 158, 139, 80,
            158, 188, 15, 202, 144, 2, 159, 144, 120, 115, 100, 200, 200,
        ],
    ),
    (
        "packages/transaction/src/keyed_fixed_request.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_fixed_request.rs"),
        [
            165, 141, 186, 239, 58, 184, 19, 71, 199, 251, 18, 25, 203, 38, 209, 197, 44, 191, 160,
            105, 186, 245, 155, 43, 133, 73, 56, 101, 203, 213, 237, 229,
        ],
    ),
    (
        "packages/transaction/src/keyed_fixed_sponsor.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_fixed_sponsor.rs"),
        [
            158, 29, 128, 153, 231, 156, 238, 189, 56, 172, 199, 97, 197, 105, 117, 1, 121, 188, 4,
            165, 200, 123, 234, 237, 238, 91, 255, 253, 166, 95, 167, 156,
        ],
    ),
    (
        "packages/transaction/src/keyed_fixed_taproot.rs",
        include_bytes!("../../../../packages/transaction/src/keyed_fixed_taproot.rs"),
        [
            200, 249, 245, 103, 32, 92, 2, 77, 40, 28, 31, 115, 159, 214, 129, 8, 234, 135, 32, 39,
            37, 151, 55, 91, 181, 86, 88, 200, 229, 16, 30, 223,
        ],
    ),
    (
        "packages/transaction/src/sponsor.rs",
        include_bytes!("../../../../packages/transaction/src/sponsor.rs"),
        [
            209, 80, 14, 189, 136, 219, 4, 1, 50, 78, 98, 116, 203, 48, 16, 183, 218, 220, 119,
            147, 153, 154, 20, 123, 218, 80, 161, 250, 115, 75, 80, 48,
        ],
    ),
    (
        "packages/transaction/src/taproot.rs",
        include_bytes!("../../../../packages/transaction/src/taproot.rs"),
        [
            196, 22, 181, 13, 154, 119, 92, 192, 55, 110, 140, 10, 251, 115, 237, 210, 81, 144,
            201, 138, 220, 183, 67, 20, 59, 9, 195, 166, 252, 171, 115, 223,
        ],
    ),
    (
        "packages/vectors/Cargo.toml",
        include_bytes!("../../../../packages/vectors/Cargo.toml"),
        [
            144, 16, 122, 216, 95, 158, 3, 32, 126, 249, 28, 252, 164, 9, 190, 213, 6, 213, 138,
            213, 150, 146, 146, 79, 38, 94, 162, 193, 99, 111, 108, 217,
        ],
    ),
    (
        "packages/vectors/src/keyed_constructor.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_constructor.rs"),
        [
            97, 55, 190, 239, 222, 120, 67, 203, 111, 164, 122, 68, 255, 224, 153, 92, 102, 85,
            136, 59, 174, 147, 53, 176, 66, 11, 130, 159, 13, 66, 169, 116,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_capture.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_capture.rs"),
        [
            5, 169, 137, 226, 10, 253, 46, 46, 247, 27, 195, 96, 205, 161, 98, 224, 178, 64, 210,
            96, 68, 11, 237, 148, 228, 59, 27, 8, 66, 121, 137, 35,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_constructor.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_constructor.rs"),
        [
            8, 94, 36, 226, 194, 205, 80, 239, 180, 14, 160, 216, 1, 218, 83, 246, 79, 248, 94, 28,
            38, 141, 40, 208, 142, 114, 208, 93, 16, 155, 99, 99,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_evidence.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_evidence.rs"),
        [
            93, 94, 111, 221, 233, 82, 173, 51, 47, 192, 65, 147, 218, 239, 44, 13, 162, 3, 164,
            57, 84, 22, 154, 30, 235, 10, 58, 165, 238, 97, 55, 198,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_history_model.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_history_model.rs"),
        [
            96, 186, 57, 136, 171, 233, 244, 147, 2, 197, 51, 158, 86, 67, 13, 216, 253, 132, 51,
            202, 110, 231, 230, 111, 183, 187, 228, 83, 64, 69, 234, 54,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_history_reader.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_history_reader.rs"),
        [
            18, 86, 188, 19, 2, 203, 222, 112, 228, 152, 147, 61, 81, 211, 236, 193, 80, 51, 75,
            37, 80, 59, 65, 226, 244, 31, 161, 170, 70, 127, 249, 5,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_history_reader/authentication.rs",
        include_bytes!(
            "../../../../packages/vectors/src/keyed_fixed_history_reader/authentication.rs"
        ),
        [
            88, 184, 14, 215, 229, 76, 2, 168, 216, 114, 194, 30, 229, 163, 182, 138, 137, 10, 125,
            31, 124, 136, 93, 252, 145, 185, 205, 174, 112, 217, 199, 67,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_history_reader/evidence.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_history_reader/evidence.rs"),
        [
            138, 39, 84, 135, 80, 114, 156, 208, 153, 219, 29, 125, 78, 88, 134, 5, 196, 146, 96,
            87, 109, 206, 169, 165, 167, 120, 142, 91, 67, 55, 48, 27,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_history_reader/fold.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_history_reader/fold.rs"),
        [
            157, 52, 252, 40, 186, 171, 146, 74, 221, 231, 218, 29, 74, 98, 243, 24, 163, 61, 230,
            65, 124, 110, 143, 99, 219, 106, 45, 181, 135, 81, 245, 226,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_history_reader/identity.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_history_reader/identity.rs"),
        [
            9, 58, 14, 80, 15, 129, 59, 96, 250, 234, 195, 93, 12, 166, 186, 132, 21, 122, 251,
            140, 17, 102, 213, 42, 76, 33, 87, 7, 141, 242, 28, 249,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_history_reader/refusal.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_history_reader/refusal.rs"),
        [
            15, 68, 73, 136, 153, 186, 101, 43, 55, 116, 14, 243, 5, 116, 220, 215, 180, 47, 54,
            21, 119, 115, 236, 10, 214, 147, 193, 91, 190, 56, 13, 12,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_manifest.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_manifest.rs"),
        [
            96, 69, 83, 247, 139, 148, 28, 228, 141, 61, 109, 232, 181, 200, 214, 113, 81, 136,
            176, 43, 8, 250, 8, 223, 137, 125, 64, 107, 153, 195, 122, 184,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_manifest/encoding.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_manifest/encoding.rs"),
        [
            178, 222, 14, 192, 223, 152, 213, 57, 174, 225, 5, 252, 99, 141, 187, 90, 101, 170,
            224, 45, 107, 243, 0, 48, 13, 36, 110, 59, 38, 167, 248, 206,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_native.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_native.rs"),
        [
            43, 44, 136, 255, 229, 143, 208, 168, 14, 55, 74, 143, 175, 243, 151, 176, 95, 109,
            176, 59, 74, 175, 115, 19, 153, 64, 54, 227, 160, 86, 16, 226,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_stack_model.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_stack_model.rs"),
        [
            248, 32, 242, 120, 190, 170, 71, 184, 142, 131, 114, 240, 219, 74, 107, 168, 116, 116,
            250, 197, 216, 78, 6, 118, 13, 62, 233, 135, 138, 225, 73, 147,
        ],
    ),
    (
        "packages/vectors/src/keyed_stack_core.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_stack_core.rs"),
        [
            35, 39, 168, 161, 45, 99, 87, 59, 244, 23, 200, 133, 119, 192, 1, 107, 166, 254, 203,
            20, 233, 207, 180, 45, 11, 180, 185, 5, 136, 161, 169, 238,
        ],
    ),
    (
        "packages/vectors/tests/keyed_fixed_capture.rs",
        include_bytes!("../../../../packages/vectors/tests/keyed_fixed_capture.rs"),
        [
            223, 168, 107, 56, 82, 68, 60, 122, 217, 227, 134, 94, 189, 16, 154, 28, 15, 248, 73,
            147, 101, 184, 15, 26, 94, 189, 97, 219, 236, 27, 99, 116,
        ],
    ),
    (
        "packages/vectors/tests/keyed_fixed_capture/native.rs",
        include_bytes!("../../../../packages/vectors/tests/keyed_fixed_capture/native.rs"),
        [
            2, 250, 241, 222, 240, 221, 99, 130, 223, 252, 214, 88, 25, 113, 71, 49, 20, 77, 184,
            122, 23, 13, 236, 0, 102, 217, 14, 124, 10, 138, 69, 144,
        ],
    ),
    (
        "scripts/elements-native-executor.py",
        include_bytes!("../../../../scripts/elements-native-executor.py"),
        [
            15, 203, 140, 112, 22, 189, 216, 146, 103, 29, 168, 230, 74, 51, 19, 51, 187, 199, 102,
            87, 6, 148, 158, 217, 95, 11, 217, 210, 247, 16, 76, 137,
        ],
    ),
    (
        "scripts/keyed-fixed-capture/capture-prepare.sh",
        include_bytes!("../../../../scripts/keyed-fixed-capture/capture-prepare.sh"),
        [
            224, 192, 248, 109, 31, 206, 41, 174, 127, 55, 176, 211, 128, 33, 71, 18, 44, 52, 131,
            90, 67, 209, 100, 198, 32, 211, 211, 125, 14, 189, 102, 222,
        ],
    ),
    (
        "scripts/keyed-fixed-capture/capture-unit.sh",
        include_bytes!("../../../../scripts/keyed-fixed-capture/capture-unit.sh"),
        [
            253, 121, 5, 60, 84, 206, 31, 48, 85, 238, 69, 165, 91, 224, 35, 150, 227, 26, 244,
            163, 158, 110, 175, 130, 101, 123, 85, 63, 252, 200, 243, 20,
        ],
    ),
    (
        "scripts/keyed-fixed-capture/run-one.sh",
        include_bytes!("../../../../scripts/keyed-fixed-capture/run-one.sh"),
        [
            57, 38, 50, 45, 190, 203, 214, 109, 79, 9, 9, 39, 189, 232, 30, 117, 130, 116, 75, 174,
            111, 40, 20, 188, 81, 150, 166, 216, 90, 209, 242, 133,
        ],
    ),
];
