# First-party crate census (ADR-014): one meson.build per crate; this
# parent composes the role groups from the per-crate lists.
subdir('architecture')
subdir('artifacts')
subdir('cli-common')
subdir('compiler')
subdir('document-stamps')
subdir('execwrap')
subdir('flatten-latex-main')
subdir('labels')
subdir('model')
subdir('realization')
subdir('tapscript')
subdir('target-elements')
subdir('target-elements-conformance')

crate_source_files = architecture_source_files + artifacts_source_files \
  + cli_common_source_files + compiler_source_files \
  + document_stamps_source_files \
  + execwrap_source_files + flatten_latex_main_source_files \
  + labels_source_files + realization_source_files \
  + tapscript_source_files + target_elements_source_files \
  + target_elements_conformance_source_files
packages_doc_files = artifacts_doc_files + cli_common_doc_files \
  + compiler_doc_files + document_stamps_doc_files \
  + execwrap_doc_files + flatten_latex_main_doc_files \
  + labels_doc_files + realization_doc_files \
  + tapscript_doc_files + target_elements_doc_files \
  + target_elements_conformance_doc_files
packages_excluded_files = artifacts_excluded_files + cli_common_excluded_files \
  + compiler_excluded_files + document_stamps_excluded_files \
  + execwrap_excluded_files \
  + flatten_latex_main_excluded_files + labels_excluded_files + model_excluded_files \
  + realization_excluded_files + tapscript_excluded_files \
  + target_elements_conformance_excluded_files \
  + target_elements_excluded_files
generated_files = model_generated_files
