const { init } = require("../near");

exports.command = 'set-owner <address>';
exports.desc = 'Set owner';
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
    .option('owner', {
      describe: 'new owner ID'
    })
    .demandOption(['signer', 'owner'])
}

exports.handler = async function (argv) {
  const { address, owner } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Setting owner to ${owner}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'set_owner',
    args: {
      new_owner_id: owner
    }
  });

  console.log('done');
}
