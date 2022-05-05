const { init } = require("../near");
const prompts = require('prompts');

exports.command = 'set-whitelist <address>';
exports.desc = 'Set whitelist';
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
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .option('whitelist', {
      describe: 'whitelist contract ID'
    })
    .demandOption(['signer', 'whitelist'])
}

exports.handler = async function (argv) {
  const { address, whitelist } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Setting whitelist to ${whitelist}`);

  const res = await prompts({
    type: 'confirm',
    name: 'confirm',
    message: 'Confirm update?'
  });
  if (!res.confirm) return;

  await signer.functionCall({
    contractId: address,
    methodName: 'set_whitelist_contract_id',
    args: {
      account_id: whitelist
    }
  });

  console.log('done');
}
