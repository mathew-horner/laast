from tree_sitter import Language

Language.build_library(
    'build/phylum-languages.so',
    [
        'vendor/tree-sitter-c-sharp',
        'vendor/tree-sitter-go',
        'vendor/tree-sitter-java',
        'vendor/tree-sitter-javascript',
        'vendor/tree-sitter-python',
        'vendor/tree-sitter-ruby',
        'vendor/tree-sitter-rust'
    ]
)
