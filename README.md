Redundant Integrated BPCS and SIS Architecture for High-Integrity Water Level Monitoring
![alt text](https://img.shields.io/badge/Language-Rust-orange.svg)

![alt text](https://img.shields.io/badge/Platform-ESP32--S3-blue.svg)

![alt text](https://img.shields.io/badge/Framework-ESP--IDF-green.svg)
📌 Project Overview
This project presents a high-integrity embedded solution for industrial water level monitoring, adhering to the core principles of the IEC 61511 safety standard. By integrating a Basic Process Control System (BPCS) and a Safety Instrumented System (SIS) into a single dual-core ESP32-S3 using the Rust programming language, the system achieves extreme reliability, memory safety, and high availability.
🔍 Background & Motivation
In high-risk industrial processes, control system failure can lead to catastrophic overflows. Traditional systems often fail due to:
Memory Corruption: Common in C/C++ (Buffer overflows, dangling pointers).
Lack of Segregation: Control logic and safety logic running on the same thread.
Sensor Failure: Relying on a single point of data.
Our Solution uses hardware-level task isolation and a Cold Redundancy Fail-over algorithm to ensure the safety layer (SIS) remains operational even if the control layer (BPCS) or primary sensors fail.
🏗 System Architecture
1. Hardware Block Diagram
The architecture leverages the Dual-Core Xtensa® LX7 processor:
Core 0 (Control Core): Manages routine BPCS logic, Alarm thresholds, and data acquisition.
Core 1 (Safety Core): An independent "Safety Kernel" dedicated to SIS logic (
≥
≥
 90% level).
Input Stage: 3x HC-SR04 Ultrasonic Sensors managed by a Fail-Over Engine.
Output Stage (Analogy):
BPCS (>12cm / 50-74%): White LED + Internal RGB Green.
ALARM (6-12cm / 75-89%): Yellow LED + Internal RGB Yellow.
SIS (<6cm / 
≥
≥
 90%): Red LED + Internal RGB Red.
2. Logic Flowchart
The system operates on a Sequential Sequential Logic:
Cold Redundancy: Only Sensor 1 (S1) is powered. S2 and S3 remain offline to save memory and power.
Health Check: If S1 fails 
→
→
 Yellow LED blinks 5x (3s) 
→
→
 Switch to S2.
Secondary Fail: If S2 fails 
→
→
 Red LED blinks 7x (3s) 
→
→
 Switch to S3.
Total Fault: If S3 fails 
→
→
 Internal RGB turns Blue (System Halt).
🛠 Tech Stack
Microcontroller: ESP32-S3 (Dual-Core).
Programming Language: Rust (no_std / esp-idf-hal).
Development Environment: Visual Studio Code with rust-analyzer.
Simulation: Wokwi / Proteus VSM.
Analysis: GNUPlot for real-time telemetry visualization.
🚀 Getting Started
Prerequisites
Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
Install ESP-RS toolchain: cargo install espup && espup install
Install flash tool: cargo install cargo-espflash
Building the Project
code
Bash
# Clone the repository
git clone https://github.com/Ahmadinf/esp32s3-rust-bpcs-sis-water-level.git

# Navigate to directory
cd esp32s3-rust-bpcs-sis-water-level

# Build the release binary
cargo build --release
Running Simulation
Open the simulation file in Proteus or Wokwi.
Point to the compiled .bin file in target/xtensa-esp32s3-espidf/release/.
Open the Serial Monitor to view real-time failsafe transitions.
📊 Results & Analysis
Simulation Logs
Data was extracted from folder-based test scenarios:
Sensor1: Successful BPCS and Alarm triggers.
Sensor2: Successful fail-over after S1 simulated failure.
Sensor3: SIS activation and final Hardware Fault (Blue RGB) verification.
GNUPlot Insights
Response Time: SIS emergency trigger latency is < 100ms.
Memory Efficiency: By utilizing cold redundancy, the RAM usage was optimized as the system avoids concurrent processing of multiple high-frequency pulse-width signals.
✨ Key Advantages
🦀 Rust Memory Safety: Zero runtime crashes from null pointers or data races.
🛡️ High Availability: 3-tier redundancy ensures the plant stays safe even with 66% sensor degradation.
🚦 Intuitive Diagnostics: Blink patterns allow operators to identify hardware issues without external debugging tools.
⚡ Hardware Isolation: SIS on Core 1 remains deterministic regardless of the network load on Core 0.
👥 The Team (Group 3)
Ahmadin Fatkhurahman - NRP: 2042241015
Ghani Raihan Syakir - NRP: 2042241036
Course: Controller Programming
Lecturer: Mr. Ahmad Radhy, S.Si., M.Si
Department: Instrumentation Engineering, Sepuluh Nopember Institute of Technology (ITS)
