const { init } = require("../near");

exports.command = 'del-manager <address>';
exports.desc = 'Remove manager';
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
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .option('manager', {
      describe: 'manager ID to remove'
    })
    .demandOption(['signer', 'manager'])
}

exports.handler = async function (argv) {
  const { address, manager } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Removing manager ${manager}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'remove_manager',
    args: {
      manager_id: manager
    }
  });

  console.log('done');
}
