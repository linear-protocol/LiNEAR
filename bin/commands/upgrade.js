const { readFileSync } = require("fs");
const nearAPI = require('near-api-js');

exports.command = 'upgrade <address>';
exports.desc = 'Upgrade contract';
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
      describe: 'New contract wasm file path',
      default: 'res/linear.wasm'
    })
}

exports.handler = async function (argv) {
  const address = argv.address;
  const code = readFileSync(argv.wasm);
  console.log(`Upgrading contract ${address}`);

  const near = await init(argv.network);
  const account = await near.account(address);

  await account.signAndSendTransaction(
    address,
    [
      nearAPI.transactions.functionCall(
        'upgrade',
        code,
        100000000000000, 
        "0"
      )
    ]
  );

  console.log('upgraded!');
}
