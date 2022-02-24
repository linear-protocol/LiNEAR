const { readFileSync } = require("fs");
const { init } = require("../near");

exports.command = 'deploy <address>';
exports.desc = 'Deploy LiNEAR contract';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address to deploy to',
      type: 'string'
    })
    .option('network', {
      describe: 'network ID',
      default: 'testnet',
      choices: ['testnet', 'mainnet']
    })
    .option('wasm', {
      describe: 'LiNEAR wasm file path',
      default: 'res/linear.wasm'
    })
};

exports.handler = async function (argv) {
  const address = argv.address;
  console.log(`Deploying ${argv.wasm} to address ${address}  ...`);

  const near = await init(argv.network);
  const account = await near.account(address);
  try {
    const result = await account.deployContract(readFileSync(argv.wasm));
    console.log(result);
    console.log('deployed!');
  } catch (err) {
    if (err && err.toString().includes("Cannot read properties of null (reading 'error_message')")) {
      console.log('deployed!');
      return;
    }
    throw err;
  }
}
