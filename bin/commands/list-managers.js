const { init } = require("../near");

exports.command = 'list-managers <address>';
exports.desc = 'List manager';
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
}

exports.handler = async function (argv) {
  const { address } = argv;
  
  const near = await init(argv.network);
  const contract = await near.account(address);

  const managers = await contract.viewFunction(address, 'get_managers');

  console.log(managers);
}
