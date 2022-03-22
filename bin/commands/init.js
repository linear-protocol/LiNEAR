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
    .demandOption(['signer', 'admin'])
    .option('signer', {
      describe: 'signer account ID to call new'
    })
    .option('admin', {
      describe: 'admin ID'
    })
};

exports.handler = async function (argv) {
  const address = argv.address;
  const adminId = argv.admin;
  console.log(`Init contract at ${address}, with adminId ${adminId}`);

  const near = await init(argv.network);
  const signer = await near.account(argv.signer);
  
  await signer.functionCall({
    contractId: address,
    methodName: 'new',
    args: {
      admin_id: adminId
    }
  });

  console.log('init done.');
}
