import 'dart:io';

import 'package:native_assets_cli/native_assets_cli.dart';
import 'package:native_toolchain_rust/native_toolchain_rust.dart';

// void main(List<String> args) async {
//   try {
//     await build(args, (BuildConfig buildConfig, BuildOutput output) async {
//       final builder = RustBuilder(
//         // The ID of native assets consists of package name and crate name.
//         package: 'dash7',
//         cratePath: '.',
//         buildConfig: buildConfig,
//       );
//       print('AAAAAAAAAAAAAAAAAAAAAAA')
//       await builder.run(output: output);
//     });
//   } catch (e) {
//     // ignore: avoid_print
//     print(e);
//     exit(1);
//   }
// }

const assetName = 'asset.txt';
final packageAssetPath = Uri.file('data/$assetName');

void main(List<String> args) async {
  await build(args, (config, output) async {
    if (config.linkModePreference == LinkModePreference.static) {
      // Simulate that this build hook only supports dynamic libraries.
      throw UnsupportedError(
        'LinkModePreference.static is not supported.',
      );
    }

    final packageName = config.packageName;
    final assetPath = config.outputDirectory.resolve(assetName);
    final assetSourcePath = config.packageRoot.resolveUri(packageAssetPath);
    if (!config.dryRun) {
      // Insert code that downloads or builds the asset to `assetPath`.
      await File.fromUri(assetSourcePath).copy(assetPath.toFilePath());

      output.addDependencies([
        assetSourcePath,
        config.packageRoot.resolve('hook/build.dart'),
      ]);
    }

    output.addAsset(
      // TODO: Change to DataAsset once the Dart/Flutter SDK can consume it.
      NativeCodeAsset(
        package: packageName,
        name: 'asset.txt',
        file: assetPath,
        linkMode: DynamicLoadingBundled(),
        os: config.targetOS,
        architecture: config.targetArchitecture,
      ),
    );
  });
}
