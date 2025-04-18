const { init, funcCall } = require("../near");
const prompts = require('prompts');
const { networkOption } = require("./common");

exports.command = 'add-manager <address> <manager>';
exports.desc = 'Propose adding a new manager to LiNEAR contract';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address',
      type: 'string'
    })
    .positional('manager', {
      describe: 'Manager account to add',
      type: 'string'
    })
    .option('network', networkOption)
    .option('signer', {
      describe: 'signer account ID to submit proposal'
    })
    .option('dao', {
      describe: 'DAO account Id'
    })
    .demandOption(['signer', 'dao'])
}

exports.handler = async function (argv) {
  const { address, manager, dao } = argv;
  const near = await init(argv.network);
  const signer = await near.account(argv.signer);

  console.log(`Propose to add manager ${manager} to linear contract ${address}`);

  const res = await prompts({
    type: 'confirm',
    name: 'confirm',
    message: 'Confirm update?'
  });
  if (!res.confirm) return;

  await funcCall(
    signer,
    dao,
    `Add manager ${manager} to linear contract`,
    address,
    'add_manager',
    {
      new_manager_id: manager
    }
  );

  console.log('done');
}
