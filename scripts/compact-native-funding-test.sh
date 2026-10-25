#!/bin/sh
# Funding contract with an in-memory transaction service; no chain operations.
set -eu
python3 - "$1/scripts/elements-native-executor.py" <<'PY'
import copy
import importlib.util
import io
import sys
import types
import unittest

spec = importlib.util.spec_from_file_location("adapter", sys.argv[1])
adapter = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = adapter
spec.loader.exec_module(adapter)


class Transaction:
    saved = {}

    def __init__(self):
        self.version = 2
        self.vin = []
        self.vout = []
        self.wit = types.SimpleNamespace(vtxinwit=[])

    def serialize(self):
        identity = str(len(self.saved)).encode()
        self.saved[identity] = copy.deepcopy(self.__dict__)
        return identity

    def deserialize(self, stream):
        self.__dict__ = copy.deepcopy(self.saved[stream.read()])


class FundingContract(unittest.TestCase):
    def setUp(self):
        self.mined = []
        self.assets = 0
        self.prepared = False
        self.reserve = bytes([1]) + bytes([0x31]) * 32
        messages = types.SimpleNamespace(
            CTransaction=Transaction,
            CTxIn=lambda point, nSequence: (point, nSequence),
            COutPoint=lambda txid, index: (txid, index),
            CTxInWitness=lambda: types.SimpleNamespace(scriptWitness=types.SimpleNamespace(stack=[])),
        )
        self.executor = types.SimpleNamespace(
            messages=messages, change=None, anyone_can_spend=b'\x51',
            anyone_can_spend_witness=b'\x00\x20' + bytes([0x32]) * 32,
            node=types.SimpleNamespace(call=self.call),
            output=lambda amount, program, field=None: (amount, program, field or self.reserve),
        )
        self.operation = adapter.OperationExecutor(self.executor)
        self.operation.prepare = self.prepare
        self.operation.mine = self.mine
        self.operation.created = self.created
        self.operation.sponsor_program = lambda: b'\x00\x14' + bytes([0x33]) * 20

    def prepare(self):
        if not self.prepared:
            self.executor.change = {'txid': '11' * 32, 'vout': 0, 'amount': 10_000_000}
            self.prepared = True

    def call(self, method, *args, **kwargs):
        if method == 'getnewaddress':
            return 'test-address'
        self.assertEqual(method, 'rawissueasset')
        self.assets += 1
        return [{'asset': ('%02x' % (0x40 + self.assets)) * 32, 'hex': args[0]}]

    def mine(self, transaction, note):
        identity = '%064x' % (len(self.mined) + 1)
        self.mined.append((identity, copy.deepcopy(transaction), note))
        return identity

    def created(self, identity, index):
        transaction = next(tx for txid, tx, _ in self.mined if txid == identity)
        amount, program, field = transaction.vout[index]
        return {'outpoint': {'txid': identity, 'vout': index},
                'asset': field[1:][::-1].hex(), 'amount_satoshis': amount, 'script': program.hex()}

    def issue(self, program=b'\x51'):
        return self.operation.issue({'issue_asset': True, 'asset': None,
                                     'output_program': program, 'outputs': 1, 'amount_per_output': 100})

    def test_first_issuance_retains_the_original_source_and_layout(self):
        result = self.issue()
        self.assertEqual(len(self.mined), 1)
        transaction = self.mined[0][1]
        self.assertEqual(transaction.vin, [((int('11' * 32, 16), 0), 0xFFFFFFFE)])
        self.assertEqual(len(transaction.vout), 4)
        self.assertEqual(transaction.vout[0][:2], (100, b'\x51'))
        self.assertEqual(transaction.vout[2][:2], (10_000_000 - adapter.ADAPTER_FEE_SATOSHIS, b'\x51'))
        self.assertEqual(transaction.wit.vtxinwit[0].scriptWitness.stack, list(adapter.ANYONE_CAN_SPEND_WITNESS_STACK))
        self.assertEqual(result['funded_outputs'], [self.created(self.mined[0][0], 0)])

    def test_repeated_issuance_retains_independent_reserves_and_programs(self):
        first = self.issue()['issued_asset']
        held = copy.deepcopy(self.operation.reserves[first])
        second = self.issue(b'\x51\x51')['issued_asset']
        self.assertNotEqual(first, second)
        self.assertEqual(self.operation.reserves[first], held)
        self.assertEqual([note for _, _, note in self.mined], ['issuance', 'issuance source', 'issuance'])
        self.assertEqual(self.mined[1][1].vout[0][1], self.executor.anyone_can_spend_witness)
        self.assertEqual(self.mined[2][1].vin[0][0], (int(self.mined[1][0], 16), 0))
        second_held = copy.deepcopy(self.operation.reserves[second])
        result = self.operation.pay({'asset': first, 'outputs': 2, 'amount_per_output': 37, 'output_program': b'\x51\x51'})
        self.assertEqual(self.operation.reserves[second], second_held)
        self.assertEqual(self.operation.reserves[first]['amount'], held['amount'] - 74)
        self.assertEqual([(coin['asset'], coin['amount_satoshis'], coin['script']) for coin in result['funded_outputs']], [(first, 37, '5151')] * 2)

    def test_signing_registration_is_confined_to_the_owned_program(self):
        asset = self.issue()['issued_asset']
        self.assertEqual(self.operation.sponsor_coins, {})
        result = self.operation.pay({'asset': asset, 'outputs': 1, 'amount_per_output': 101,
                                     'output_program': self.operation.sponsor_program()})
        coin = result['funded_outputs'][0]
        key = '%s:%d' % (coin['outpoint']['txid'], coin['outpoint']['vout'])
        self.assertEqual(self.operation.sponsor_coins, {key: {
            'value_field': adapter.explicit_value_field(101), 'asset': asset}})
        self.operation.pay({'asset': asset, 'outputs': 1, 'amount_per_output': 17, 'output_program': b'\x51'})
        self.assertEqual(list(self.operation.sponsor_coins), [key])


unittest.main(argv=[sys.argv[0]])
PY
