const { init } = require("../near");
const { networkOption } = require("./common");

exports.command = 'add-manager <address>';
exports.desc = 'Add manager';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address',
      type: 'string'
    })
    .option('network', networkOption)
    .option('signer', {
      describe: 'signer account Id to call contract'
    })
    .option('manager', {
      describe: 'new manager ID'
    })
    .demandOption(['signer', 'manager'])
}

exports.handler = async function (argv) {
  const { address, manager } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Adding manager ${manager}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'add_manager',
    args: {
      new_manager_id: manager
    }
  });

  console.log('done');
}
