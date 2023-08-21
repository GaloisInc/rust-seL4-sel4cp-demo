# Simple test script
import pexpect


# Start QEMU
child = pexpect.spawn('make run',encoding='utf-8')
fout = open('log.txt','w')
child.logfile = fout
# Wait for the prompt
child.expect('banscii>',timeout=2)
# Try hello world
child.sendline('Hello\r')
child.expect('PING',timeout=2)
child.sendline('World\r')
child.expect('PONG',timeout=2)
# Escape sequence
child.sendcontrol('A')
child.send('x')
# Termination confirmation
child.expect('QEMU: Terminated',timeout=1)

print("Test Succeeded!")
