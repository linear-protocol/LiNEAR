const { init } = require("../near");

exports.command = 'list-operators <address>';
exports.desc = 'List operator';
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

  const operators = await contract.viewFunction(address, 'get_operators');

  console.log(operators);
}
