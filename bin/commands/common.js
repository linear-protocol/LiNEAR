const prompts = require('prompts');

exports.networkOption = {
  describe: 'network ID',
  default: 'testnet',
  choices: ['testnet', 'mainnet', 'localnet']
};

exports.doubleCheck = async () => {
  const res = await prompts({
    type: 'toggle',
    name: 'value',
    message: 'Confirm?',
    initial: true,
    active: 'yes',
    inactive: 'no'
  });
  if (!res.value) process.exit(1);
}
