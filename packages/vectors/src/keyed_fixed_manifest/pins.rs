//! Initial S5 whole-file pins. The pin table is represented in the encoded
//! inventory itself; hashing this table into itself would be circular.

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
        "packages/target-elements-conformance/src/constructor/tree.rs",
        include_bytes!("../../../../packages/target-elements-conformance/src/constructor/tree.rs"),
        [
            54, 79, 12, 224, 130, 76, 135, 97, 22, 80, 145, 120, 108, 25, 85, 94, 182, 235, 68, 86,
            128, 42, 178, 39, 152, 91, 52, 150, 116, 148, 140, 34,
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
        "packages/vectors/Cargo.toml",
        include_bytes!("../../../../packages/vectors/Cargo.toml"),
        [
            210, 205, 104, 73, 158, 92, 218, 86, 242, 12, 120, 250, 56, 154, 252, 229, 21, 102,
            187, 125, 120, 246, 185, 220, 167, 205, 51, 234, 70, 244, 136, 100,
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
        "packages/vectors/src/keyed_fixed_constructor.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_constructor.rs"),
        [
            8, 94, 36, 226, 194, 205, 80, 239, 180, 14, 160, 216, 1, 218, 83, 246, 79, 248, 94, 28,
            38, 141, 40, 208, 142, 114, 208, 93, 16, 155, 99, 99,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_manifest.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_manifest.rs"),
        [
            247, 189, 17, 104, 74, 228, 83, 141, 41, 69, 9, 185, 189, 164, 104, 1, 105, 51, 227,
            77, 127, 10, 37, 138, 29, 243, 130, 76, 128, 177, 148, 231,
        ],
    ),
    (
        "packages/vectors/src/keyed_fixed_manifest/encoding.rs",
        include_bytes!("../../../../packages/vectors/src/keyed_fixed_manifest/encoding.rs"),
        [
            102, 242, 94, 152, 223, 132, 43, 34, 197, 35, 79, 241, 109, 246, 222, 164, 31, 23, 133,
            50, 247, 74, 86, 142, 118, 95, 199, 65, 198, 253, 177, 28,
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
];
