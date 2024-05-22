import 'package:dash7/main.dart';
import 'package:test/test.dart';

import 'util.dart';

Future<void> main({bool skipLibInit = false}) async {
  await init(skipLibInit: skipLibInit);

  test('varint: constructor', () async {
      expect(VarInt(value: 1, ceil: false).value, 1);
      expect(VarInt(value: 32, ceil: false).value, 32);
      expect(VarInt(value: 507904, ceil: false).value, 507904);
  });

  test('varint: decompress', () async {
        expect(0, VarInt.decompress(exponent: 0, mantissa: 0).value);
        expect(4, VarInt.decompress(exponent: 1, mantissa: 1).value);
        expect(32, VarInt.decompress(exponent: 2, mantissa: 2).value);
        expect(192, VarInt.decompress(exponent: 3, mantissa: 3).value);
        expect(507904, VarInt.decompress(exponent: 7, mantissa: 31).value);

  });

}
