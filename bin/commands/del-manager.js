const { init, funcCall } = require("../near");
const prompts = require('prompts');
const { networkOption } = require("./common");

exports.command = 'del-manager <address> <manager>';
exports.desc = 'Propose removing a manager from LiNEAR contract';
exports.builder = yargs => {
  yargs
    .positional('address', {
      describe: 'Contract address',
      type: 'string'
    })
    .positional('manager', {
      describe: 'Manager account to remove',
      type: 'string'
    })
    .option('network', networkOption)
    .option('signer', {
      describe: 'signer account ID to call new'
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

  console.log(`Propose to remove manager ${manager} from linear contract ${address}`);

  const res = await prompts({
    type: 'confirm',
    name: 'confirm',
    message: 'Confirm update?'
  });
  if (!res.confirm) return;

  await funcCall(
    signer,
    dao,
    `Remove manager ${manager} from linear contract`,
    address,
    'remove_manager',
    {
      manager_id: manager
    }
  );

  console.log('done');
}
