import graphviz
import hashlib
import os
import subprocess

from queue import Queue
from tree_sitter import Language, Parser

def build_languages_object(languages):
    object = {}
    for language in languages:
        object[language] = Language(LANGUAGES_FILE, language)
    return object

LANGUAGES_FILE = 'build/phylum-languages.so'
LANGUAGES = build_languages_object(
    ['c_sharp', 'go', 'java', 'javascript', 'python', 'ruby', 'rust']
)

EXTENSION_LANGUAGES = {
    'cs': 'c_sharp',
    'go': 'go',
    'java': 'java',
    'js': 'javascript',
    'py': 'python',
    'rb': 'ruby',
    'rs': 'rust',
}

BLACKLISTED_NODE_TYPES = set([
    # Punctuation varies heavily amongst programming languages and can
    # artificially deflate similarity across languages.
    '(', ')', '.', ';', '!', '[', ']', '{', '}', '"', '\'', ':',
])

TYPE_MAP = {
    'compilation_unit': 'unit',
    'module': 'unit',
    'program': 'unit',
    'source_file': 'unit',

    'formal_parameters': 'parameters',
    'method_parameters': 'parameters',
    'parameter_list': 'parameters',

    'body_statement': 'block',
    'statement_block': 'block',

    'function_declaration': 'function_definition',
    'function_item': 'function_definition',
    'local_function_statement': 'function_definition',
    'method': 'function_definition',
    'method_declaration': 'function_definition',
}

class Laast:
    def __init__(self, language, code):
        parser = Parser()
        parser.set_language(LANGUAGES[language])
        code_bytes = bytes(code, "utf8")
        ts_ast = parser.parse(code_bytes)
        self.ast = LaastNode.from_tree_sitter_ast(ts_ast)
        self.language = language
        self.checksum = hashlib.sha256(code_bytes).hexdigest()

    def from_file(filename):
        _, extension = filename.split('.', 1)
        if extension not in EXTENSION_LANGUAGES:
            raise ValueError(f'"{extension}" is not a supported extension')
        language = EXTENSION_LANGUAGES[extension]
        with open(filename) as f:
            code = f.read()
            return Laast(language, code)

class LaastNode:
    def __init__(self, properties):
        self.properties = properties
        self.children = []

    def from_tree_sitter_node(ts_node):
        # In some cases, I've seen this type be a single space character.
        # No matter the situation, we don't care about whitespace, so a strip
        # here suffices.
        type = ts_node.type.strip()
        
        if type in BLACKLISTED_NODE_TYPES:
            return None

        node = LaastNode({ 'type': TYPE_MAP.get(type, type) })
        for child in ts_node.children:
            child_laast_node = LaastNode.from_tree_sitter_node(child)
            if child_laast_node is not None:
                node.children.append(child_laast_node)

        return node

    def from_tree_sitter_ast(ts_ast):
        return LaastNode.from_tree_sitter_node(ts_ast.root_node)

def graphviz_render_laast(laast):
    graph = graphviz.Digraph(f'AST-{laast.language}-{laast.checksum}', directory='graphs')
    current_id = 1
    q = Queue()
    q.put((None, laast.ast))    

    while not q.empty():
        parent_id, ast_node = q.get()

        # Assign this node an id for references (the GraphViz API requires a
        # string here, so we just stringify our autoincremented id)
        node_id = str(current_id)
        current_id += 1

        # Insert this node into the graph, labelling it as the AST node type
        # that it is (module, function, etc)
        graph.node(node_id, ast_node.properties['type'])

        # All nodes except the root node *should* have a parent, so we need to
        # draw an edge from parent -> child
        if parent_id is not None:
            graph.edge(parent_id, node_id)

        # Traverse all the children of the current node
        for child in ast_node.children:
            q.put((node_id, child))
        
    graph.view()

def read_example_asts(example_name):
    directory = f'examples/{example_name}'
    return [
        Laast.from_file(f'{directory}/{filename}') for filename in os.listdir(directory)
    ]

def calculate_similarity_stats(asts):
    ted_stats = calculate_tree_edit_distance_stats(asts)
    return {
        'edit_distance': ted_stats
    }

def calculate_tree_edit_distance_stats(asts):
    teds = []
    for i in range(0, len(asts)):
        for j in range(i + 1, len(asts)):
            ted = calculate_tree_edit_distance(asts[i], asts[j])
            teds.append(ted)

    return {
        'avg': sum(teds) // len(teds),
        'min': min(teds),
        'max': max(teds),
    }

def calculate_tree_edit_distance(ast1, ast2):
    ast1_bracket_notation = encode_laast_node_into_bracket_notation(ast1.ast)
    ast2_bracket_notation = encode_laast_node_into_bracket_notation(ast2.ast)
    command = subprocess.run([
        './vendor/tree-similarity/build/ted',
        'string',
        f'"{ast1_bracket_notation}"',
        f'"{ast2_bracket_notation}"',
    ], capture_output=True)
    output = command.stdout.decode('utf8')
    output_lines = output.strip().split('\n')
    return int(output_lines[-1].split(':')[1])

def encode_laast_node_into_bracket_notation(node):
    # For the purposes of this tree similarity algorithm, we'll put the node
    # type as the node label.
    type = node.properties['type']
    other_props = {k: v for k, v in node.properties.items() if k != 'type'}
    bn = '{'
    bn += '\{['
    bn += type
    bn += '],\{'
    bn += ','.join([f'"{k}":"{v}"' for k, v in other_props])
    bn += '\}\}'
    bn += ''.join([encode_laast_node_into_bracket_notation(child) for child in node.children])
    bn += '}'
    return bn

if __name__ == '__main__':
    # laast = Laast.from_file('examples/hello-world/rust.rs')
    # bn = encode_laast_node_into_bracket_notation(laast.ast)
    # print(bn)
    # graphviz_render_laast(laast)
    laasts = read_example_asts('hello-world')
    stats = calculate_similarity_stats(laasts)
    print(stats)
    for laast in laasts:
        graphviz_render_laast(laast)
