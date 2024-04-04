import 'package:app/messages/counter.pb.dart';
import 'package:flutter/material.dart';
import './messages/generated.dart';

void main() async {
  await initializeRust();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'axum-rinf-flutter-example',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const MyHomePage(title: 'axum-rinf-flutter-example'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            StreamBuilder(
                stream: Counter.rustSignalStream,
                builder: (context, snapshot) {
                  final rustSignal = snapshot.data;
                  if (rustSignal == null) {
                    return const Text("Nothing received yet");
                  }
                  final counter = rustSignal.message;
                  final currentNumebr = counter.number;
                  return Text(
                    currentNumebr.toString(),
                    style: const TextStyle(fontSize: 40),
                  );
                }),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: () {
          SetCounter(counter: 0).sendSignalToRust(null);
        },
        tooltip: 'Reset counter number',
        child: const Icon(Icons.restart_alt),
      ),
    );
  }
}
