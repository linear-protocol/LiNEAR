const { init } = require("../near");

exports.command = 'init <address>';
exports.desc = 'Init mock staking pool contract';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address',
      type: 'string'
    })
    .option('network', {
      describe: 'network ID',
      default: 'testnet',
      choices: ['testnet', 'mainnet']
    })
    .demandOption(['signer'])
    .option('signer', {
      describe: 'signer account ID to call new'
    })
};

exports.handler = async function (argv) {
  const address = argv.address;
  console.log(`Init contract at ${address}`);

  const near = await init(argv.network);
  const signer = await near.account(argv.signer);
  
  await signer.functionCall({
    contractId: address,
    methodName: 'new',
    args: {}
  });

  console.log('init done.');
}
