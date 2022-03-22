const { init } = require("../near");

exports.command = 'set-admin <address>';
exports.desc = 'Set admin';
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
    .option('admin', {
      describe: 'new admin ID'
    })
    .demandOption(['signer', 'admin'])
}

exports.handler = async function (argv) {
  const { address, admin } = argv;
  
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Setting admin to ${admin}`);

  await signer.functionCall({
    contractId: address,
    methodName: 'set_admin',
    args: {
      new_admin_id: admin
    }
  });

  console.log('done');
}
