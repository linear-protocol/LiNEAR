const { init } = require("../near");

exports.command = 'init <address>';
exports.desc = 'Init LiNEAR contract';
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
    .demandOption(['signer', 'owner'])
    .option('signer', {
      describe: 'signer account ID to call new'
    })
    .option('owner', {
      describe: 'owner ID'
    })
};

exports.handler = async function (argv) {
  const address = argv.address;
  const ownerId = argv.owner;
  console.log(`Init contract at ${address}, with ownerId ${ownerId}`);

  const near = await init(argv.network);
  const signer = await near.account(argv.signer);
  
  await signer.functionCall({
    contractId: address,
    methodName: 'new',
    args: {
      owner_id: ownerId
    }
  });

  console.log('init done.');
}
