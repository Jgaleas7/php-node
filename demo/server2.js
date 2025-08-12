import { cwd } from 'node:process';
import { join } from 'node:path';
import { strictEqual } from 'node:assert';
import { run_python_script } from '../index.js'; // The new function name

const docroot = join(cwd(), 'demo');
const script_path = 'hello.py'; // Your new Python script
const expected_output = 'Hello, from Python!';

// Create a simple Python script
const script_content = `print('${expected_output}')`;
require('node:fs').writeFileSync(join(docroot, script_path), script_content);

try {
    const output = run_python_script(script_path, docroot);
    strictEqual(output.trim(), expected_output);
    console.log(`Success! The Python script output: "${output.trim()}"`);
} catch (err) {
    console.error('Error running Python script:', err);
}
